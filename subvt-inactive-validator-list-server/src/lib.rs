//! Subscribes to the inactive validator list data on Redis and publishes the data
//! through WebSocket pub/sub.

use anyhow::Context;
use async_trait::async_trait;
use bus::Bus;
use jsonrpsee::ws_server::{RpcModule, WsServerBuilder, WsStopHandle};
use lazy_static::lazy_static;
use log::{debug, error, warn};
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_types::{
    crypto::AccountId,
    subvt::{
        InactiveValidator, InactiveValidatorDiff, InactiveValidatorListUpdate,
        InactiveValidatorSummary,
    },
};
use tokio::task::JoinHandle;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Clone, Debug)]
pub enum BusEvent {
    Update(InactiveValidatorListUpdate),
    Error,
}

#[derive(Default)]
pub struct InactiveValidatorListServer;

impl InactiveValidatorListServer {
    pub async fn run_rpc_server(
        validator_map: &Arc<RwLock<HashMap<AccountId, InactiveValidator>>>,
        bus: &Arc<Mutex<Bus<BusEvent>>>,
    ) -> anyhow::Result<(JoinHandle<()>, WsStopHandle)> {
        let rpc_ws_server = WsServerBuilder::default()
            .max_request_body_size(u32::MAX)
            .build(format!(
                "{}:{}",
                CONFIG.rpc.host, CONFIG.rpc.inactive_validator_list_port
            ))
            .await?;
        let mut rpc_module = RpcModule::new(());
        let validator_map = validator_map.clone();
        let bus = bus.clone();
        rpc_module.register_subscription(
            "subscribe_inactive_validator_list",
            "unsubscribe_inactive_validator_list",
            move |_params, mut sink, _| {
                debug!("New subscription.");
                let mut bus_receiver = bus.lock().unwrap().add_rx();
                {
                    let validator_summaries: Vec<InactiveValidatorSummary> = {
                        let validator_map = validator_map.read().unwrap();
                        validator_map.iter().map(|value| value.1.into()).collect()
                    };
                    let update = InactiveValidatorListUpdate {
                        insert: validator_summaries,
                        ..Default::default()
                    };
                    let _ = sink.send(&update);
                }
                std::thread::spawn(move || loop {
                    if let Ok(update) = bus_receiver.recv() {
                        match update {
                            BusEvent::Update(update) => {
                                let send_result = sink.send(&update);
                                if let Err(error) = send_result {
                                    debug!("Subscription closed. {:?}", error);
                                    return;
                                } else {
                                    debug!("Published diff.");
                                }
                            }
                            BusEvent::Error => {
                                return;
                            }
                        }
                    }
                });
                Ok(())
            },
        )?;
        let stop_handle = rpc_ws_server.stop_handle();
        let join_handle = tokio::spawn(rpc_ws_server.start(rpc_module));
        Ok((join_handle, stop_handle))
    }
}

#[async_trait]
impl Service for InactiveValidatorListServer {
    async fn run(&'static self) -> anyhow::Result<()> {
        let last_finalized_block_number = 0;
        let bus = Arc::new(Mutex::new(Bus::new(100)));
        let validator_map = Arc::new(RwLock::new(HashMap::<AccountId, InactiveValidator>::new()));
        let prefix = format!("subvt:{}:inactive_validators", CONFIG.substrate.chain);

        let redis_client = redis::Client::open(CONFIG.redis.url.as_str()).context(format!(
            "Cannot connect to Redis at URL {}.",
            CONFIG.redis.url
        ))?;
        let mut pub_sub_connection = redis_client.get_connection()?;
        let mut pub_sub = pub_sub_connection.as_pubsub();
        pub_sub.subscribe(format!(
            "subvt:{}:inactive_validators:publish:finalized_block_number",
            CONFIG.substrate.chain
        ))?;
        let mut data_connection = redis_client.get_connection()?;
        let (server_join_handle, server_stop_handle) =
            InactiveValidatorListServer::run_rpc_server(&validator_map, &bus).await?;

        let error: anyhow::Error = loop {
            let message = pub_sub.get_message();
            if let Err(error) = message {
                break error.into();
            }
            let payload = message.unwrap().get_payload();
            if let Err(error) = payload {
                break error.into();
            }
            let finalized_block_number: u64 = payload.unwrap();
            if last_finalized_block_number == finalized_block_number {
                warn!(
                    "Skip duplicate finalized block #{}.",
                    finalized_block_number
                );
            }
            debug!("New finalized block #{}.", finalized_block_number);

            let validator_addresses: HashSet<String> = redis::cmd("SMEMBERS")
                .arg(format!("{}:addresses", prefix))
                .query(&mut data_connection)
                .context("Can't read inactive validator addresses from Redis.")?;
            debug!(
                "Got {} validator addresses. Checking for changes...",
                validator_addresses.len()
            );
            let mut update = InactiveValidatorListUpdate {
                finalized_block_number: Some(finalized_block_number),
                ..Default::default()
            };
            {
                // find the ones to remove
                let validator_map = validator_map.read().unwrap();
                for validator_account_id in validator_map.keys() {
                    if !validator_addresses.contains(&validator_account_id.to_string()) {
                        update.remove_ids.push(validator_account_id.clone());
                    }
                }
            }
            {
                // remove
                let mut validator_map = validator_map.write().unwrap();
                for remove_id in &update.remove_ids {
                    validator_map.remove(remove_id);
                }
            }
            let mut new_validators: Vec<InactiveValidator> = Vec::new();
            let mut validator_updates: Vec<InactiveValidatorDiff> = Vec::new();
            {
                // update/insert
                let validator_map = validator_map.read().unwrap();
                for validator_address in validator_addresses {
                    let validator_account_id = AccountId::from_str(&validator_address).unwrap();
                    let prefix = format!("{}:validator:{}", prefix, validator_address);
                    if let Some(validator) = validator_map.get(&validator_account_id) {
                        // check hash, if different, fetch, calculate and add to list
                        let hash = {
                            let mut hasher = DefaultHasher::new();
                            validator.hash(&mut hasher);
                            hasher.finish()
                        };
                        let db_hash: u64 = redis::cmd("GET")
                            .arg(format!("{}:hash", prefix))
                            .query(&mut data_connection)
                            .context("Can't read inactive validator hash from Redis.")?;
                        if hash != db_hash {
                            debug!("Hash changed for {}.", validator_address);
                            let validator_json_string: String = redis::cmd("GET")
                                .arg(prefix)
                                .query(&mut data_connection)
                                .context("Can't read inactive validator addresses from Redis.")?;
                            let db_validator: InactiveValidator =
                                serde_json::from_str(&validator_json_string)?;
                            let db_validator_summary: InactiveValidatorSummary =
                                InactiveValidatorSummary::from(&db_validator);
                            let validator_summary: InactiveValidatorSummary = validator.into();
                            update
                                .update
                                .push(db_validator_summary.get_diff(&validator_summary));
                            validator_updates.push(validator.get_diff(&db_validator));
                        }
                    } else {
                        let validator_json_string: String = redis::cmd("GET")
                            .arg(prefix)
                            .query(&mut data_connection)
                            .context("Can't read inactive validator addresses from Redis.")?;
                        let validator: InactiveValidator =
                            serde_json::from_str(&validator_json_string)?;
                        let validator_summary = InactiveValidatorSummary::from(&validator);
                        update.insert.push(validator_summary);
                        new_validators.push(validator);
                    }
                }
            }
            {
                let mut validator_map = validator_map.write().unwrap();
                for diff in validator_updates {
                    let validator = validator_map.get_mut(&diff.account.id).unwrap();
                    validator.apply_diff(&diff);
                }
                for validator in new_validators {
                    validator_map.insert(validator.account.id.clone(), validator);
                }
            }
            debug!(
                "Completed checks. Remove {} validators. {} new validators. {} updated validators.",
                update.remove_ids.len(),
                update.insert.len(),
                update.update.len(),
            );
            {
                let mut bus = bus.lock().unwrap();
                bus.broadcast(BusEvent::Update(update));
                debug!("Update published to the bus.");
            }
        };
        error!("{:?}", error);
        {
            let mut bus = bus.lock().unwrap();
            bus.broadcast(BusEvent::Error);
        }
        debug!("Stopping RPC server...");
        server_stop_handle.clone().stop().await?;
        server_join_handle
            .await
            .expect("Server can't be shut down.");
        debug!("RPC server fully stopped.");
        Err(error)
    }
}

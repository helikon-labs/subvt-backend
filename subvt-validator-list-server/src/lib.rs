//! Validator list WebSocket server. Operates on the configured port. Serves the inactive validator
//! list if the `--inactive` command-line flag is provided at startup, otherwise serves the active
//! validator list.
//!
//! Supports two RPC methods: `subscribe_validatorList` and `unsubscribe_validatorList`.
//! Gives the complete list at first connection, then publishes only the changed validators' fields
//! after each update from `subvt-validator-list-updater`.
use anyhow::Context;
use async_trait::async_trait;
use bus::Bus;
use clap::{arg, Command};
use futures_util::StreamExt as _;
use jsonrpsee::ws_server::{RpcModule, WsServerBuilder, WsServerHandle};
use lazy_static::lazy_static;
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_types::{
    crypto::AccountId,
    subvt::{ValidatorDetails, ValidatorDetailsDiff, ValidatorListUpdate, ValidatorSummary},
};

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Clone, Debug)]
pub enum BusEvent {
    Update(ValidatorListUpdate),
    Error,
}

#[derive(Default)]
pub struct ValidatorListServer;

#[allow(clippy::cognitive_complexity)]
fn is_inactive() -> bool {
    Command::new("SubVT Validator List Server")
        .version("0.1.0")
        .author("Kutsal Kaan Bilgin <kutsal@helikon.io>")
        .about("Serves the active or inactive validator list for the SubVT app.")
        .arg(arg!(-i --inactive "Active list is served by default. Use this flag to serve the inactive list."))
        .get_matches()
        .is_present("inactive")
}

impl ValidatorListServer {
    pub async fn run_rpc_server(
        host: &str,
        port: u16,
        validator_map: &Arc<RwLock<HashMap<AccountId, ValidatorDetails>>>,
        bus: &Arc<Mutex<Bus<BusEvent>>>,
    ) -> anyhow::Result<WsServerHandle> {
        let rpc_ws_server = WsServerBuilder::default()
            .max_request_body_size(u32::MAX)
            .build(format!("{}:{}", host, port))
            .await?;
        let mut rpc_module = RpcModule::new(());
        let validator_map = validator_map.clone();
        let bus = bus.clone();
        rpc_module.register_subscription(
            "subscribe_validatorList",
            "subscribe_validatorList",
            "unsubscribe_validatorList",
            move |_params, mut sink, _| {
                log::info!("New subscription.");
                metrics::subscription_count().inc();
                let mut bus_receiver = bus.lock().unwrap().add_rx();
                {
                    let validator_summaries: Vec<ValidatorSummary> = {
                        let validator_map = validator_map.read().unwrap();
                        validator_map.iter().map(|value| value.1.into()).collect()
                    };
                    let update = ValidatorListUpdate {
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
                                    metrics::subscription_count().dec();
                                    log::info!("Subscription closed. {:?}", error);
                                    return;
                                } else {
                                    log::debug!("Published diff.");
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
        Ok(rpc_ws_server.start(rpc_module)?)
    }
}

#[async_trait(?Send)]
impl Service for ValidatorListServer {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            if is_inactive() {
                CONFIG.metrics.inactive_validator_list_server_port
            } else {
                CONFIG.metrics.active_validator_list_server_port
            },
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        let is_active_list = !is_inactive();
        // init metrics
        metrics::init(if is_active_list {
            "subvt_active_validator_list_server"
        } else {
            "subvt_inactive_validator_list_server"
        });
        let mut last_finalized_block_number = 0;
        let bus = Arc::new(Mutex::new(Bus::new(100)));
        let validator_map = Arc::new(RwLock::new(HashMap::<AccountId, ValidatorDetails>::new()));

        let redis_client = redis::Client::open(CONFIG.redis.url.as_str()).context(format!(
            "Cannot connect to Redis at URL {}.",
            CONFIG.redis.url
        ))?;
        let mut pubsub_connection = redis_client.get_async_connection().await?.into_pubsub();
        pubsub_connection
            .subscribe(format!(
                "subvt:{}:validators:publish:finalized_block_number",
                CONFIG.substrate.chain
            ))
            .await?;
        let mut data_connection = redis_client.get_connection()?;
        metrics::subscription_count().set(0);
        let server_stop_handle = ValidatorListServer::run_rpc_server(
            &CONFIG.rpc.host,
            if is_active_list {
                CONFIG.rpc.active_validator_list_port
            } else {
                CONFIG.rpc.inactive_validator_list_port
            },
            &validator_map,
            &bus,
        )
        .await?;

        let mut pubsub_stream = pubsub_connection.on_message();
        let error: anyhow::Error = 'outer: loop {
            let maybe_message = pubsub_stream.next().await;
            let payload = if let Some(message) = maybe_message {
                message.get_payload()
            } else {
                continue;
            };
            if let Err(error) = payload {
                break error.into();
            }
            let finalized_block_number: u64 = payload.unwrap();
            if last_finalized_block_number == finalized_block_number {
                log::warn!(
                    "Skip duplicate finalized block #{}.",
                    finalized_block_number
                );
                continue 'outer;
            }
            log::info!("New finalized block #{}.", finalized_block_number);
            metrics::target_finalized_block_number().set(finalized_block_number as i64);
            let prefix = format!(
                "subvt:{}:validators:{}:{}",
                CONFIG.substrate.chain,
                finalized_block_number,
                if is_active_list { "active" } else { "inactive" }
            );
            let validator_account_ids: HashSet<String> = redis::cmd("SMEMBERS")
                .arg(format!("{}:account_id_set", prefix))
                .query(&mut data_connection)
                .context("Can't read validator account ids from Redis.")?;
            log::info!(
                "Got {} validator account ids. Checking for changes...",
                validator_account_ids.len()
            );
            let mut update = ValidatorListUpdate {
                finalized_block_number: Some(finalized_block_number),
                ..Default::default()
            };
            {
                // find the ones to remove
                let validator_map = validator_map.read().unwrap();
                for validator_account_id in validator_map.keys() {
                    if !validator_account_ids.contains(&validator_account_id.to_string()) {
                        update.remove_ids.push(*validator_account_id);
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
            let mut new_validators: Vec<ValidatorDetails> = Vec::new();
            let mut validator_updates: Vec<ValidatorDetailsDiff> = Vec::new();
            {
                // update/insert
                let validator_map = validator_map.read().unwrap();
                for validator_account_id in validator_account_ids {
                    let validator_account_id = AccountId::from_str(&validator_account_id).unwrap();
                    let prefix = format!("{}:validator:{}", prefix, validator_account_id);
                    if let Some(validator) = validator_map.get(&validator_account_id) {
                        // check hash, if different, fetch, calculate and add to list
                        let summary_hash = {
                            let mut hasher = DefaultHasher::new();
                            ValidatorSummary::from(validator).hash(&mut hasher);
                            hasher.finish()
                        };
                        let db_summary_hash: u64 = redis::cmd("GET")
                            .arg(format!("{}:summary_hash", prefix))
                            .query(&mut data_connection)
                            .context("Can't read validator summary hash from Redis.")?;
                        if summary_hash != db_summary_hash {
                            log::info!("Summary hash changed for {}.", validator_account_id);
                            let validator_json_string: String = redis::cmd("GET")
                                .arg(prefix)
                                .query(&mut data_connection)
                                .context("Can't read validator JSON string (1) from Redis.")?;
                            let db_validator: ValidatorDetails =
                                serde_json::from_str(&validator_json_string)?;
                            let db_validator_summary: ValidatorSummary =
                                ValidatorSummary::from(&db_validator);
                            let validator_summary: ValidatorSummary = validator.into();
                            update
                                .update
                                .push(validator_summary.get_diff(&db_validator_summary));
                            validator_updates.push(validator.get_diff(&db_validator));
                        }
                    } else {
                        let validator_json_string: String = redis::cmd("GET")
                            .arg(&prefix)
                            .query(&mut data_connection)
                            .context(format!(
                                "Can't read validator JSON string (2) from Redis :: {}",
                                &prefix
                            ))?;
                        let validator_deser_result: serde_json::error::Result<ValidatorDetails> =
                            serde_json::from_str(&validator_json_string);
                        match validator_deser_result {
                            Ok(validator) => {
                                let validator_summary = ValidatorSummary::from(&validator);
                                update.insert.push(validator_summary);
                                new_validators.push(validator);
                            }
                            Err(error) => {
                                break 'outer error.into();
                            }
                        }
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
                    validator_map.insert(validator.account.id, validator);
                }
            }
            log::info!(
                "Completed checks. Remove {} validators. {} new validators. {} updated validators.",
                update.remove_ids.len(),
                update.insert.len(),
                update.update.len(),
            );
            {
                let mut bus = bus.lock().unwrap();
                bus.broadcast(BusEvent::Update(update));
                log::info!("Update published to the bus.");
            }
            metrics::processed_finalized_block_number().set(finalized_block_number as i64);
            last_finalized_block_number = finalized_block_number;
        };
        log::error!("{:?}", error);
        {
            let mut bus = bus.lock().unwrap();
            bus.broadcast(BusEvent::Error);
        }
        log::info!("Stopping RPC server...");
        server_stop_handle.stop()?;
        log::info!("RPC server fully stopped.");
        Err(error)
    }
}

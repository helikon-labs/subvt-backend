//! Validator details WebSocket server. Operates on the configured port.
//!
//! Supports two RPC methods: `subscribe_validatorDetails` and `unsubscribe_validatorDetails`.
//! `subscribe_validatorDetails` accepts a single parameter: 0x-prefixed hex-encoded account id
//! of the validator. Gives the complete details at first connection, then publishes only the
//! changed fields after each update from `subvt-validator-list-updater`.
use anyhow::Context;
use async_trait::async_trait;
use bus::Bus;
use futures_util::StreamExt as _;
use jsonrpsee::ws_server::{RpcModule, WsServerBuilder, WsServerHandle};
use lazy_static::lazy_static;
use redis::RedisResult;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_types::crypto::AccountId;
use subvt_types::subvt::{ValidatorDetails, ValidatorDetailsDiff};

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
    static ref LAST_FINALIZED_BLOCK_NUMBER: AtomicU64 = AtomicU64::new(0);
}

#[derive(Clone, Debug)]
pub enum BusEvent {
    NewFinalizedBlock(u64),
    Error,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ValidatorDetailsUpdate {
    finalized_block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validator_details: Option<ValidatorDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validator_details_update: Option<ValidatorDetailsDiff>,
}

#[derive(Default)]
pub struct ValidatorDetailsServer;

impl ValidatorDetailsServer {
    fn fetch_validator_details(
        account_id: &str,
        redis_client: &redis::Client,
    ) -> anyhow::Result<ValidatorDetails> {
        let mut connection = redis_client.get_connection()?;
        let active_validator_key = format!(
            "subvt:{}:validators:{}:active:validator:{}",
            CONFIG.substrate.chain,
            LAST_FINALIZED_BLOCK_NUMBER.load(Ordering::SeqCst),
            account_id,
        );
        let active_validator_json_string_result: RedisResult<String> = redis::cmd("GET")
            .arg(active_validator_key)
            .query(&mut connection);
        let validator_json_string = match active_validator_json_string_result {
            Ok(validator_json_string) => validator_json_string,
            Err(_) => {
                let inactive_validator_key = format!(
                    "subvt:{}:validators:{}:inactive:validator:{}",
                    CONFIG.substrate.chain,
                    LAST_FINALIZED_BLOCK_NUMBER.load(Ordering::SeqCst),
                    account_id,
                );
                redis::cmd("GET")
                    .arg(inactive_validator_key)
                    .query(&mut connection)?
            }
        };
        Ok(serde_json::from_str(&validator_json_string)?)
    }

    pub async fn run_rpc_server(
        host: &str,
        port: u16,
        redis_client: &redis::Client,
        bus: Arc<Mutex<Bus<BusEvent>>>,
    ) -> anyhow::Result<WsServerHandle> {
        let rpc_ws_server = WsServerBuilder::default()
            .max_request_body_size(u32::MAX)
            .build(format!("{}:{}", host, port))
            .await?;
        let mut rpc_module = RpcModule::new(());
        let redis_client = redis_client.clone();
        let data_connection = Arc::new(RwLock::new(redis_client.get_connection()?));
        rpc_module.register_subscription(
            "subscribe_validatorDetails",
            "subscribe_validatorDetails",
            "unsubscribe_validatorDetails",
            move |params, pending, _| {
                let account_id = match params.one::<String>() {
                    Ok(param) => {
                        if let Ok(account_id) = AccountId::from_str(&param) {
                            account_id
                        } else {
                            pending.reject(jsonrpsee::types::error::ErrorCode::InvalidParams);
                            return;
                        }
                    }
                    Err(_) => {
                        pending.reject(jsonrpsee::types::error::ErrorCode::InvalidParams);
                        return;
                    }
                };
                let mut sink = match pending.accept() {
                    Some(sink) => sink,
                    _ => {
                        log::warn!("Cannot accept new subscription: connection closed.");
                        return;
                    }
                };
                log::info!("New subscription {}.", account_id);
                metrics::subscription_count().inc();
                let mut validator_details = {
                    let validator_details = match ValidatorDetailsServer::fetch_validator_details(
                        &account_id.to_string(),
                        &redis_client,
                    ) {
                        Ok(validator_details) => validator_details,
                        Err(error) => {
                            log::error!("Error while fetching validator details: {:?}", error);
                            let error_message = "Error while fetching validator details. Please make sure you are sending a valid validator account id.".to_string();
                            let _ = sink.send(&error_message);
                            return;
                        }
                    };
                    let _ = sink.send(&ValidatorDetailsUpdate {
                        finalized_block_number: Some(LAST_FINALIZED_BLOCK_NUMBER.load(Ordering::SeqCst)),
                        validator_details: Some(validator_details.clone()),
                        validator_details_update: None
                    });
                    validator_details
                };
                let mut bus_receiver = bus.lock().unwrap().add_rx();
                let data_connection = data_connection.clone();
                std::thread::spawn(move || {
                    loop {
                        if let Ok(update) = bus_receiver.recv() {
                            if sink.is_closed() {
                                log::info!("Subscription connection closed.");
                                metrics::subscription_count().dec();
                                return;
                            }
                            match update {
                                BusEvent::NewFinalizedBlock(finalized_block_number) => {
                                    let active_validator_storage_key_prefix =  format!(
                                        "subvt:{}:validators:{}:active:validator:{}",
                                        CONFIG.substrate.chain,
                                        finalized_block_number,
                                        account_id,
                                    );
                                    let inactive_validator_storage_key_prefix =  format!(
                                        "subvt:{}:validators:{}:inactive:validator:{}",
                                        CONFIG.substrate.chain,
                                        finalized_block_number,
                                        account_id,
                                    );
                                    let hash = {
                                        let mut hasher = DefaultHasher::new();
                                        validator_details.hash(&mut hasher);
                                        hasher.finish()
                                    };
                                    let mut data_connection = data_connection.write().unwrap();
                                    let (validator_storage_key_prefix, db_hash) = if let Ok(db_hash) = redis::cmd("GET")
                                        .arg(format!(
                                            "{}:hash",
                                            active_validator_storage_key_prefix,
                                        ))
                                        .query::<u64>(&mut *data_connection) {
                                        (active_validator_storage_key_prefix, db_hash)
                                    } else if let Ok(db_hash) = redis::cmd("GET")
                                        .arg(format!(
                                            "{}:hash",
                                            inactive_validator_storage_key_prefix,
                                        ))
                                        .query::<u64>(&mut *data_connection) {
                                        (inactive_validator_storage_key_prefix, db_hash)
                                    } else {
                                        log::error!(
                                            "Validator {} not found.",
                                            account_id
                                        );
                                        return;
                                    };
                                    let update = if hash != db_hash {
                                        let validator_json_string_result = redis::cmd("GET")
                                            .arg(&validator_storage_key_prefix)
                                            .query::<String>(&mut *data_connection);
                                        let validator_json_string = match validator_json_string_result {
                                            Ok(validator_json_string) => validator_json_string,
                                            Err(error) => {
                                                log::error!(
                                                    "Error while fetching validator JSON string for storage key {}: {:?}",
                                                    validator_storage_key_prefix,
                                                    error
                                                );
                                                return;
                                            }
                                        };
                                        let db_validator_details_result =
                                            serde_json::from_str::<ValidatorDetails>(&validator_json_string);
                                        let db_validator_details = match db_validator_details_result {
                                            Ok(db_validator_details) => db_validator_details,
                                            Err(error) => {
                                                log::error!(
                                                    "Error while deserializing validator details for storage key {}: {:?}",
                                                    validator_storage_key_prefix,
                                                    error
                                                );
                                                return;
                                            }
                                        };
                                        let update = ValidatorDetailsUpdate {
                                            finalized_block_number: Some(finalized_block_number),
                                            validator_details: None,
                                            validator_details_update: Some(validator_details.get_diff(&db_validator_details)),
                                        };
                                        validator_details = db_validator_details;
                                        update
                                    } else {
                                        ValidatorDetailsUpdate {
                                            finalized_block_number: Some(finalized_block_number),
                                            validator_details: None,
                                            validator_details_update: None,
                                        }
                                    };
                                    let send_result = sink.send(&update);
                                    match send_result {
                                        Err(error) => {
                                            log::warn!("Error during publish: {:?}", error);
                                            metrics::subscription_count().dec();
                                            return;
                                        }
                                        Ok(is_successful) => {
                                            if is_successful {
                                                log::debug!("Diff published.");
                                            } else {
                                                log::info!("Publish failed. Closing connection.");
                                                metrics::subscription_count().dec();
                                                return;
                                            }
                                        }
                                    }
                                }
                                BusEvent::Error => {
                                    log::error!("Bus update receive error.");
                                    return;
                                }
                            }
                        }
                    }
                });
            },
        )?;
        Ok(rpc_ws_server.start(rpc_module)?)
    }
}

#[async_trait(?Send)]
impl Service for ValidatorDetailsServer {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.validator_details_server_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        // let last_finalized_block_number = Arc::new(AtomicU64::new(0));
        let bus = Arc::new(Mutex::new(Bus::new(100)));
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
        metrics::subscription_count().set(0);
        let server_stop_handle = ValidatorDetailsServer::run_rpc_server(
            &CONFIG.rpc.host,
            CONFIG.rpc.validator_details_port,
            &redis_client,
            bus.clone(),
        )
        .await?;
        let mut pubsub_stream = pubsub_connection.on_message();
        let error: anyhow::Error = loop {
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
            if LAST_FINALIZED_BLOCK_NUMBER.load(Ordering::SeqCst) == finalized_block_number {
                log::warn!(
                    "Skip duplicate finalized block #{}.",
                    finalized_block_number
                );
                continue;
            }
            log::info!("New finalized block #{}.", finalized_block_number);
            metrics::current_finalized_block_number().set(finalized_block_number as i64);
            {
                let mut bus = bus.lock().unwrap();
                bus.broadcast(BusEvent::NewFinalizedBlock(finalized_block_number));
                log::debug!("Update published to the bus.");
            }
            LAST_FINALIZED_BLOCK_NUMBER.store(finalized_block_number, Ordering::SeqCst);
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

//! Subscribes to the network status data on Redis and publishes the data through
//! websocket pub/sub.
#![warn(clippy::disallowed_types)]
use anyhow::Context;
use async_trait::async_trait;
use bus::Bus;
use futures_util::StreamExt as _;
use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
use lazy_static::lazy_static;
use redis::aio::Connection;
use std::sync::{Arc, Mutex, RwLock};
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_types::subvt::{NetworkStatus, NetworkStatusDiff, NetworkStatusUpdate};

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Clone, Debug)]
pub enum BusEvent {
    NewBlock(Box<NetworkStatusDiff>),
    Error,
}

#[derive(Default)]
pub struct NetworkStatusServer;

impl NetworkStatusServer {
    async fn read_current_network_status(
        connection: &mut Connection,
    ) -> anyhow::Result<NetworkStatus> {
        let key = format!("subvt:{}:network_status", CONFIG.substrate.chain);
        let status_json_string: String = redis::cmd("GET")
            .arg(key)
            .query_async(connection)
            .await
            .context("Can't read network status from Redis.")?;
        let status: NetworkStatus = serde_json::from_str(&status_json_string)
            .context("Can't deserialize network status json.")?;
        Ok(status)
    }

    async fn run_rpc_server(
        current_status: &Arc<RwLock<NetworkStatus>>,
        bus: &Arc<Mutex<Bus<BusEvent>>>,
    ) -> anyhow::Result<ServerHandle> {
        let rpc_ws_server = ServerBuilder::default()
            .build(format!(
                "{}:{}",
                CONFIG.rpc.host, CONFIG.rpc.network_status_port
            ))
            .await?;
        let mut rpc_module = RpcModule::new(());
        let current_status = current_status.clone();
        let bus = bus.clone();
        rpc_module.register_subscription(
            "subscribe_networkStatus",
            "subscribe_networkStatus",
            "unsubscribe_networkStatus",
            move |_params, mut sink, _| {
                log::info!("New subscription.");
                metrics::subscription_count().inc();
                let mut bus_receiver = bus.lock().unwrap().add_rx();
                {
                    let current_status = current_status.read().unwrap();
                    if current_status.best_block_number != 0 {
                        let update = NetworkStatusUpdate {
                            network: CONFIG.substrate.chain.clone(),
                            status: Some(current_status.clone()),
                            diff_base_block_number: None,
                            diff: None,
                        };
                        let _ = sink.send(&update);
                    }
                }
                std::thread::spawn(move || loop {
                    if let Ok(status_diff) = bus_receiver.recv() {
                        if sink.is_closed() {
                            log::info!("Subscription connection closed.");
                            metrics::subscription_count().dec();
                            return;
                        }
                        match status_diff {
                            BusEvent::NewBlock(status_diff) => {
                                let update = NetworkStatusUpdate {
                                    network: CONFIG.substrate.chain.clone(),
                                    status: None,
                                    diff_base_block_number: None,
                                    diff: Some(*status_diff.clone()),
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

/// Service implementation.
#[async_trait(?Send)]
impl Service for NetworkStatusServer {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.network_status_server_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        let bus = Arc::new(Mutex::new(Bus::new(100)));
        let current_status = Arc::new(RwLock::new(NetworkStatus::default()));
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str()).context(format!(
            "Cannot connect to Redis at URL {}.",
            CONFIG.redis.url
        ))?;

        let mut pubsub_connection = redis_client.get_async_connection().await?.into_pubsub();
        pubsub_connection
            .subscribe(format!(
                "subvt:{}:network_status:publish:best_block_number",
                CONFIG.substrate.chain
            ))
            .await?;
        let mut data_connection = redis_client.get_async_connection().await?;
        metrics::subscription_count().set(0);
        let server_stop_handle = NetworkStatusServer::run_rpc_server(&current_status, &bus).await?;

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
            let best_block_number: u64 = payload.unwrap();
            {
                let current_status = current_status.read().unwrap();
                if current_status.best_block_number == best_block_number {
                    log::warn!("Skip duplicate best block #{}.", best_block_number);
                    continue;
                }
            }
            log::info!("New best block #{}.", best_block_number);
            metrics::target_best_block_number().set(best_block_number as i64);
            match NetworkStatusServer::read_current_network_status(&mut data_connection).await {
                Ok(new_status) => {
                    {
                        let current_status = current_status.read().unwrap();
                        if current_status.best_block_number != 0 {
                            let diff = current_status.get_diff(&new_status);
                            let mut bus = bus.lock().unwrap();
                            bus.broadcast(BusEvent::NewBlock(Box::new(diff)));
                            metrics::processed_best_block_number().set(best_block_number as i64);
                        }
                    }
                    let mut current_status = current_status.write().unwrap();
                    *current_status = new_status;
                }
                Err(error) => {
                    break error;
                }
            }
        };
        log::error!("{:?}", error);
        {
            let mut bus = bus.lock().unwrap();
            bus.broadcast(BusEvent::Error);
        }
        log::info!("Stop RPC server.");
        server_stop_handle.clone().stop()?;
        log::info!("RPC server stopped fully.");
        Err(error)
    }
}

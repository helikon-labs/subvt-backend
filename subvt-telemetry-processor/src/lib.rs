//! Connects to the WebSocket feed stream of the given Telemetry and stores the feed data in
//! the time series database (TimeScaleDB on PostgreSQL). Can be configured to connect to the
//! W3F or Polkadot Telemetry servers.
#![warn(clippy::disallowed_types)]
use anyhow::Context;
use async_lock::Mutex;
use async_trait::async_trait;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures::StreamExt;
use lazy_static::lazy_static;
use rustc_hash::FxHashMap as HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_types::telemetry::{FeedMessage, NodeDetails, NodeLocation};

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct TelemetryProcessor;

impl TelemetryProcessor {
    #[allow(clippy::cognitive_complexity)]
    async fn process_feed_message(
        postgres: &PostgreSQLNetworkStorage,
        node_map: &Mutex<HashMap<u64, NodeDetails>>,
        feed_message: &FeedMessage,
    ) -> anyhow::Result<()> {
        match feed_message {
            FeedMessage::Version(version) => {
                log::debug!("Version: {version}.");
            }
            FeedMessage::BestBlock {
                block_number,
                timestamp,
                avg_block_time,
            } => {
                log::debug!("Best block: {block_number} {timestamp} {avg_block_time:?}.",);
                metrics::best_block_number().set(*block_number as i64);
                postgres
                    .update_best_block_number(*block_number, *timestamp, *avg_block_time)
                    .await?;
            }
            FeedMessage::BestFinalized {
                block_number,
                block_hash,
            } => {
                log::debug!("Finalized block: {block_number} {block_hash}.");
                metrics::finalized_block_number().set(*block_number as i64);
                postgres
                    .update_finalized_block_number(
                        *block_number,
                        &format!("0x{}", block_hash.trim_start_matches("0x").to_uppercase()),
                    )
                    .await?;
            }
            FeedMessage::AddedNode {
                node_id,
                node_details,
                stats: _stats,
                io: _io,
                hardware: _hardware,
                block_details: _block_details,
                location,
                startup_time,
            } => {
                log::debug!("Add node #{node_id} :: {node_details:?}.");
                let mut map = node_map.lock().await;
                if map.insert(*node_id, *node_details.clone()).is_none() {
                    metrics::node_count().inc();
                }
                postgres
                    .save_node(*node_id, node_details, *startup_time, location)
                    .await?;
            }
            FeedMessage::RemovedNode { node_id } => {
                log::debug!("Removed node #{node_id}.");
                let mut map = node_map.lock().await;
                if map.remove(node_id).is_some() {
                    metrics::node_count().dec();
                }
                postgres.remove_node(*node_id).await?;
            }
            FeedMessage::LocatedNode {
                node_id,
                latitude,
                longitude,
                city,
            } => {
                let location = NodeLocation(*latitude, *longitude, city.clone());
                postgres.update_node_location(*node_id, &location).await?;
            }
            FeedMessage::NodeImportedBlock {
                node_id,
                block_details,
            } => {
                log::trace!(
                    "Node #{} imported block #{}.",
                    node_id,
                    block_details.block_number
                );
                postgres
                    .update_node_best_block(
                        *node_id,
                        block_details.block_number,
                        &block_details.block_hash,
                    )
                    .await?;
            }
            FeedMessage::NodeFinalizedBlock {
                node_id,
                block_number,
                block_hash,
            } => {
                log::trace!("Node #{node_id} finalized block #{block_number}.");
                postgres
                    .update_node_finalized_block(*node_id, *block_number, block_hash)
                    .await?;
            }
            FeedMessage::NodeStatsUpdate { node_id, stats } => {
                log::trace!("Node #{node_id} status {stats:?}.");
                if let Err(error) = postgres.save_node_stats(*node_id, stats).await {
                    log::error!("Error while saving node stats: {error:?}");
                }
            }
            FeedMessage::NodeHardware { node_id, hardware } => {
                log::trace!("Node #{node_id} hardware {hardware:?}.");
                if hardware.0.len() != hardware.1.len() || hardware.1.len() != hardware.2.len() {
                    log::warn!(
                        "Invalid node network stats data. Timestamp [{}], download bandwidth [{}] and upload bandwidth [{}] vectors are not of equal lengths.",
                        hardware.2.len(),
                        hardware.1.len(),
                        hardware.0.len(),
                    );
                } else if let Err(error) =
                    postgres.save_node_network_stats(*node_id, hardware).await
                {
                    log::error!("Error while saving node network stats: {error:?}");
                }
            }
            FeedMessage::TimeSync { time } => {
                log::debug!("Time sync :: {time}");
            }
            FeedMessage::AddedChain {
                name,
                genesis_hash,
                node_count,
            } => {
                log::debug!("Added chain {name} {genesis_hash} {node_count}");
            }
            FeedMessage::RemovedChain { genesis_hash } => {
                log::debug!("Removed chain {genesis_hash}");
            }
            FeedMessage::SubscribedTo { genesis_hash } => {
                log::debug!("Subscribed to chain {genesis_hash}");
            }
            FeedMessage::UnsubscribedFrom { genesis_hash } => {
                log::debug!("Unsubscribed from chain {genesis_hash}");
            }
            FeedMessage::Pong { message } => {
                log::trace!("Pong :: {message}");
            }
            FeedMessage::StaleNode { node_id } => {
                log::trace!("Stale node #{node_id}.");
            }
            FeedMessage::NodeIOUpdate { node_id, io } => {
                log::trace!("IO update #{node_id} :: {io:?}");
            }
            _ => (),
        }
        Ok(())
    }

    async fn receive_messages(tx: Sender<Vec<FeedMessage>>) -> anyhow::Result<()> {
        // connect to Telemetry feed
        let (mut ws_stream, _) = connect_async(&CONFIG.telemetry.websocket_url)
            .await
            .context("Failed to connect")?;
        log::debug!("Telemetry server websocket handshake has been successfully completed.");
        ws_stream
            .send(Message::text(format!(
                "subscribe:{}",
                CONFIG.substrate.chain_genesis_hash
            )))
            .await?;
        log::debug!("Subscribed to the chain.");
        // receiver thread
        let error = loop {
            let message_result = match ws_stream.next().await {
                Some(message_result) => message_result,
                None => {
                    log::warn!("None message received. Try to receive next message.");
                    continue;
                }
            };
            let message = match message_result {
                Ok(message) => message,
                Err(error) => {
                    log::error!("Error while receiving Telemetry message: {error:?}");
                    break error.into();
                }
            };
            let bytes = &message.into_data();
            let feed_messages = match FeedMessage::from_bytes(bytes) {
                Ok(feed_messages) => feed_messages,
                Err(error) => {
                    log::error!("Error while decoding Telemetry feed message: {error:?}");
                    break error;
                }
            };
            tx.send(feed_messages)?;
        };
        Err(error)
    }

    async fn process_messages(
        node_map: Mutex<HashMap<u64, NodeDetails>>,
        rx: Receiver<Vec<FeedMessage>>,
    ) -> anyhow::Result<()> {
        let postgres =
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
        for messages in rx {
            for message in messages {
                TelemetryProcessor::process_feed_message(&postgres, &node_map, &message).await?;
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl Service for TelemetryProcessor {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.telemetry_processor_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!("Running the Telemetry processor.");
        metrics::init();
        metrics::node_count().set(0);
        let (tx, rx) = mpsc::channel();
        tokio::spawn(async move {
            loop {
                let tx = tx.clone();
                if let Err(error) = TelemetryProcessor::receive_messages(tx).await {
                    log::error!("Error while receiving feed messages: {error:?}");
                }
            }
        });
        let node_map: Mutex<HashMap<u64, NodeDetails>> = Default::default();
        if let Err(error) = TelemetryProcessor::process_messages(node_map, rx).await {
            log::error!("Error while processing feed messages: {error:?}");
        }
        Ok(())
    }
}

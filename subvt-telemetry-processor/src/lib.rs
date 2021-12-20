// https://github.com/paritytech/substrate-telemetry/blob/master/backend/test_utils/src/feed_message_de.rs

use anyhow::Context;
use async_lock::Mutex;
use async_trait::async_trait;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_types::telemetry::{FeedMessage, NodeDetails, NodeLocation};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct TelemetryProcessor;

impl TelemetryProcessor {
    async fn process_feed_message(
        postgres: &PostgreSQLNetworkStorage,
        node_map: &Mutex<HashMap<u64, NodeDetails>>,
        feed_message: &FeedMessage,
    ) -> anyhow::Result<()> {
        match feed_message {
            FeedMessage::Version(version) => {
                trace!("Version: {}.", version);
            }
            FeedMessage::BestBlock {
                block_number,
                timestamp,
                avg_block_time,
            } => {
                // PERSIST
                trace!(
                    "Best block: {} {} {:?}.",
                    block_number,
                    timestamp,
                    avg_block_time
                );
            }
            FeedMessage::BestFinalized {
                block_number,
                block_hash,
            } => {
                // PERSIST
                trace!("Best finalized: {} {}.", block_number, block_hash);
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
                trace!("Add node #{} :: {:?}.", node_id, node_details);
                let mut map = node_map.lock().await;
                map.insert(*node_id, *node_details.clone());
                postgres
                    .save_node(*node_id, node_details, *startup_time, location)
                    .await?;
            }
            FeedMessage::RemovedNode { node_id } => {
                trace!("Removed node #{}.", node_id);
                let mut map = node_map.lock().await;
                map.remove(node_id);
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
                trace!(
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
                trace!("Node #{} finalized block #{}.", node_id, block_number);
                postgres
                    .update_node_finalized_block(*node_id, *block_number, block_hash)
                    .await?;
            }
            FeedMessage::NodeStatsUpdate { node_id, stats } => {
                trace!("Node #{} status {:?}.", node_id, stats);
                if let Err(error) = postgres.save_node_stats(*node_id, stats).await {
                    error!("Error while saving node stats: {:?}", error);
                }
            }
            FeedMessage::NodeHardware { node_id, hardware } => {
                trace!("Node #{} hardware {:?}.", node_id, hardware);
                if hardware.0.len() != hardware.1.len() || hardware.1.len() != hardware.2.len() {
                    warn!(
                        "Invalid node network stats data. Timestamp [{}], download bandwidth [{}] and upload bandwidth [{}] vectors are not of equal lengths.",
                        hardware.2.len(),
                        hardware.1.len(),
                        hardware.0.len(),
                    );
                } else if let Err(error) =
                    postgres.save_node_network_stats(*node_id, hardware).await
                {
                    error!("Error while saving node network stats: {:?}", error);
                }
            }
            FeedMessage::TimeSync { time } => {
                trace!("Time sync :: {}", time);
            }
            FeedMessage::AddedChain {
                name,
                genesis_hash,
                node_count,
            } => {
                trace!("Added chain {} {} {}", name, genesis_hash, node_count);
            }
            FeedMessage::RemovedChain { genesis_hash } => {
                trace!("Removed chain {}", genesis_hash);
            }
            FeedMessage::SubscribedTo { genesis_hash } => {
                trace!("Subscribed to chain {}", genesis_hash);
            }
            FeedMessage::UnsubscribedFrom { genesis_hash } => {
                trace!("Unsubscribed from chain {}", genesis_hash);
            }
            FeedMessage::Pong { message } => {
                trace!("Pong :: {}", message);
            }
            FeedMessage::StaleNode { node_id } => {
                trace!("Stale node #{}.", node_id);
            }
            FeedMessage::NodeIOUpdate { node_id, io } => {
                trace!("IO update #{} :: {:?}", node_id, io);
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
        debug!("Telemetry server websocket handshake has been successfully completed.");
        ws_stream
            .send(Message::text(format!(
                "subscribe:{}",
                CONFIG.substrate.chain_genesis_hash
            )))
            .await?;
        debug!("Subscribed to the chain.");
        // receiver thread
        let error = loop {
            let message_result = match ws_stream.next().await {
                Some(message_result) => message_result,
                None => {
                    warn!("None message received. Try to receive next message.");
                    continue;
                }
            };
            let message = match message_result {
                Ok(message) => message,
                Err(error) => {
                    error!("Error while receiving Telemetry message: {:?}", error);
                    break error.into();
                }
            };
            let bytes = &message.into_data();
            let feed_messages = match FeedMessage::from_bytes(bytes) {
                Ok(feed_messages) => feed_messages,
                Err(error) => {
                    error!("Error while decoding Telemetry feed message: {:?}", error);
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
    async fn run(&'static self) -> anyhow::Result<()> {
        info!("Running the Telemetry processor.");
        let (tx, rx) = mpsc::channel();
        let a1 = tokio::spawn(async move {
            loop {
                let tx = tx.clone();
                if let Err(error) = TelemetryProcessor::receive_messages(tx).await {
                    error!("Error while receiving feed messages: {:?}", error);
                }
            }
        });
        let node_map: Mutex<HashMap<u64, NodeDetails>> = Default::default();
        let b1 = tokio::spawn(async move {
            if let Err(error) = TelemetryProcessor::process_messages(node_map, rx).await {
                error!("Error while processing feed messages: {:?}", error);
            }
        });
        info!("Receiving and processing messages.");
        let (a1_result, b1_result) = tokio::join!(a1, b1,);
        a1_result?;
        b1_result?;
        Ok(())
    }
}

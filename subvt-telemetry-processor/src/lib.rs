// https://github.com/paritytech/substrate-telemetry/blob/master/backend/test_utils/src/feed_message_de.rs

use crate::types::FeedMessage;
use anyhow::Context;
use async_trait::async_trait;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use std::sync::mpsc::{self, Receiver, Sender};
use subvt_config::Config;
use subvt_service_common::Service;

mod types;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct TelemetryProcessor;

impl TelemetryProcessor {
    fn process_feed_message(feed_message: &FeedMessage) {
        match feed_message {
            FeedMessage::Version(version) => {
                println!("Version: {}.", version);
            }
            FeedMessage::BestBlock {
                block_number,
                timestamp,
                avg_block_time,
            } => {
                println!(
                    "Best block: {} {} {:?}.",
                    block_number, timestamp, avg_block_time
                );
            }
            FeedMessage::BestFinalized {
                block_number,
                block_hash,
            } => {
                println!("Best finalized: {} {}.", block_number, block_hash);
            }
            FeedMessage::AddedNode {
                node_id,
                node,
                stats: _stats,
                io: _io,
                hardware: _hardware,
                block_details: _block_details,
                location: _location,
                startup_time: _startup_time,
            } => {
                println!("Added node #{} :: {:?}.", node_id, node);
            }
            FeedMessage::RemovedNode { node_id } => {
                println!("Removed node #{}.", node_id);
            }
            FeedMessage::LocatedNode {
                node_id,
                latitude: _,
                longitude: _,
                city: _,
            } => {
                println!("Located node #{}.", node_id);
            }
            FeedMessage::NodeImportedBlock {
                node_id,
                block_details,
            } => {
                println!(
                    "Node #{} imported block #{}.",
                    node_id, block_details.block_number
                );
            }
            FeedMessage::NodeFinalizedBlock {
                node_id,
                block_number,
                block_hash: _,
            } => {
                println!("Node #{} finalized block #{}.", node_id, block_number);
            }
            FeedMessage::NodeStatsUpdate { node_id, stats } => {
                println!("Node #{} status {:?}.", node_id, stats);
            }
            FeedMessage::NodeHardware { node_id, hardware } => {
                println!("Node #{} hardware {:?}.", node_id, hardware);
            }
            FeedMessage::TimeSync { time } => {
                println!("Time sync :: {}", time);
            }
            FeedMessage::AddedChain {
                name,
                genesis_hash,
                node_count,
            } => {
                println!("Added chain {} {} {}", name, genesis_hash, node_count);
            }
            FeedMessage::RemovedChain { genesis_hash } => {
                println!("Removed chain {}", genesis_hash);
            }
            FeedMessage::SubscribedTo { genesis_hash } => {
                println!("Subscribed to chain {}", genesis_hash);
            }
            FeedMessage::UnsubscribedFrom { genesis_hash } => {
                println!("Unsubscribed from chain {}", genesis_hash);
            }
            FeedMessage::Pong { message } => {
                println!("Pong :: {}", message);
            }
            FeedMessage::StaleNode { node_id } => {
                println!("Stale node #{}.", node_id);
            }
            FeedMessage::NodeIOUpdate { node_id, io } => {
                println!("IO update #{} :: {:?}", node_id, io);
            }
            _ => (),
        }
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

    async fn process_messages(rx: Receiver<Vec<FeedMessage>>) -> anyhow::Result<()> {
        rx.iter().for_each(|feed_messages| {
            feed_messages
                .iter()
                .for_each(TelemetryProcessor::process_feed_message)
        });
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
                let _a_result = TelemetryProcessor::receive_messages(tx).await;
            }
        });
        let b1 = tokio::spawn(async move {
            let _b_result = TelemetryProcessor::process_messages(rx).await;
        });
        let (a1_result, b1_result) = tokio::join!(a1, b1,);
        a1_result?;
        b1_result?;
        Ok(())
    }
}

// https://github.com/paritytech/substrate-telemetry/blob/master/backend/test_utils/src/feed_message_de.rs

use crate::types::FeedMessage;
use anyhow::Context;
use async_trait::async_trait;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{debug, info};
use subvt_config::Config;
use subvt_service_common::Service;

mod types;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct TelemetryProcessor;

impl TelemetryProcessor {}

#[async_trait(?Send)]
impl Service for TelemetryProcessor {
    async fn run(&'static self) -> anyhow::Result<()> {
        info!("Running the Telemetry processor.");
        // connect to Telemetry feed
        let (mut ws_stream, _) = connect_async("wss://telemetry.w3f.community/feed")
            .await
            .context("Failed to connect")?;
        debug!("Telemetry server websocket handshake has been successfully completed.");
        let _msg = ws_stream
            .next()
            .await
            .ok_or("didn't receive anything")
            .unwrap()
            .unwrap();
        ws_stream
            .send(Message::text(format!(
                "subscribe:{}",
                CONFIG.substrate.chain_genesis_hash
            )))
            .await?;
        /*
        // println!("#2 Received: {}", msg.into_text()?);
        let msg = ws_stream
            .next()
            .await
            .ok_or("didn't receive anything")
            .unwrap()
            .unwrap();
        println!("#3 Received: {}", msg.into_text()?);
        */
        let msg = ws_stream
            .next()
            .await
            .ok_or("didn't receive anything")
            .unwrap()
            .unwrap();
        println!("#1 Received: {}", msg.into_text()?);
        let msg = ws_stream
            .next()
            .await
            .ok_or("didn't receive anything")
            .unwrap()
            .unwrap();
        println!("#2 Received: {}", msg.into_text()?);

        loop {
            let msg = ws_stream
                .next()
                .await
                .ok_or("didn't receive anything")
                .unwrap()
                .unwrap();
            let v1 = &msg.into_data();
            let feed_messages = FeedMessage::from_bytes(v1)?;
            for feed_message in feed_messages {
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
        }
    }
}

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
        let msg = ws_stream
            .next()
            .await
            .ok_or("didn't receive anything")
            .unwrap()
            .unwrap();
        println!("#0 Received: {}", msg.into_text()?);
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
            println!("RECEIVED: {}", msg.clone().into_text()?);
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
                        println!("Added node #: {} {:?}.", node_id, node);
                    }
                    _ => (),
                }
            }
        }
    }
}

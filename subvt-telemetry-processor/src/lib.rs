use anyhow::Context;
use async_trait::async_trait;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{debug, info};
use serde_json::value::RawValue;
use subvt_config::Config;
use subvt_service_common::Service;

mod types;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct TelemetryProcessor;

impl TelemetryProcessor {}

/*
struct FeedMessage(u8);
struct AddNodeMessage(u8);

impl<'de> serde::Deserialize<'de> for FeedMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value.get(0).and_then(Value::as_u64).unwrap() {
            3 => println!("ADD NODE"),
            unsupported_type => println!("unsupported type {:?}", unsupported_type),
        }
        // println!("value :: {:?}", value);
        Ok(FeedMessage(3))
    }
}

 */

pub enum FeedMessage {
    BestBlock {
        block_number: u64,
        timestamp: u64,
        avg_block_time: Option<u64>,
    },
}

#[async_trait(?Send)]
impl Service for TelemetryProcessor {
    async fn run(&'static self) -> anyhow::Result<()> {
        info!("Running the Telemetry processor.");

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

        loop {
            let msg = ws_stream
                .next()
                .await
                .ok_or("didn't receive anything")
                .unwrap()
                .unwrap();
            let v1 = &msg.into_data();
            let v: Vec<&RawValue> = serde_json::from_slice(v1)?;
            // https://github.com/paritytech/substrate-telemetry/blob/a4069e4b3d6f74d1cacdd3f2af50cb7105943cd0/backend/test_utils/src/feed_message_de.rs
            for raw_keyval in v.chunks(2) {
                let raw_key = raw_keyval[0];
                let raw_val = raw_keyval[1];
                let action: u8 = serde_json::from_str(raw_key.get())?;
                if action == 1 {
                    let (block_number, timestamp, avg_block_time) =
                        serde_json::from_str(raw_val.get())?;
                    let _message = FeedMessage::BestBlock {
                        block_number,
                        timestamp,
                        avg_block_time,
                    };
                    println!("BEST BLOCK {}", block_number);
                }
            }
        }
    }
}

use anyhow::Context;
use serde::Deserialize;
use serde_json::value::RawValue;

#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeStats {
    peer_count: u64,
    queued_tx_count: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeIO(
    // state cache size
    Vec<f32>,
);

#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeHardware(
    // upload bandwidth
    Vec<f64>,
    // download bandwidth
    Vec<f64>,
    // timestamps
    Vec<f64>,
);

#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeLocation(
    // latitude
    f32,
    // longitude
    f32,
    // city
    String,
);

#[derive(Debug, PartialEq)]
pub struct NodeDetails {
    name: String,
    implementation: String,
    version: String,
    stash_address: Option<String>,
    network_id: Option<String>,
}

#[derive(Deserialize)]
pub struct Block {
    _block_hash: String,
    _block_height: u64,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct BlockDetails {
    block_number: u64,
    block_hash: String,
    block_time: u64,
    // block timestamp
    block_timestamp: u64,
    propagation_time: Option<u64>,
}

#[derive(Debug, PartialEq)]
pub enum FeedMessage {
    Version(usize),
    BestBlock {
        block_number: u64,
        timestamp: u64,
        avg_block_time: Option<u64>,
    },
    BestFinalized {
        block_number: u64,
        block_hash: String,
    },
    AddedNode {
        node_id: usize,
        node: Box<NodeDetails>,
        stats: NodeStats,
        io: NodeIO,                  // can't losslessly deserialize
        hardware: Box<NodeHardware>, // can't losslessly deserialize
        block_details: BlockDetails,
        location: Option<NodeLocation>,
        startup_time: Option<u64>,
    },
    UnknownValue {
        action: u8,
        value: String,
    },
}

impl FeedMessage {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Vec<FeedMessage>> {
        let v: Vec<&RawValue> = serde_json::from_slice(bytes)?;
        let mut feed_messages = vec![];
        for raw_key_value in v.chunks(2) {
            let raw_key = raw_key_value[0];
            let raw_value = raw_key_value[1];
            let action: u8 = serde_json::from_str(raw_key.get())?;
            let msg = FeedMessage::decode(action, raw_value).with_context(|| {
                format!("Failed to decode feed message with action {}.", action)
            })?;
            feed_messages.push(msg);
        }
        Ok(feed_messages)
    }

    fn decode(action: u8, raw_value: &RawValue) -> anyhow::Result<FeedMessage> {
        let feed_message = match action {
            // version
            0 => {
                let version = serde_json::from_str(raw_value.get())?;
                FeedMessage::Version(version)
            }
            // best block
            1 => {
                let (block_number, timestamp, avg_block_time) =
                    serde_json::from_str(raw_value.get())?;
                FeedMessage::BestBlock {
                    block_number,
                    timestamp,
                    avg_block_time,
                }
            }
            // best finalized
            2 => {
                let (block_number, block_hash) = serde_json::from_str(raw_value.get())?;
                FeedMessage::BestFinalized {
                    block_number,
                    block_hash,
                }
            }
            // added node
            3 => {
                let (
                    node_id,
                    (name, implementation, version, stash_address, network_id),
                    (peer_count, queued_tx_count),
                    (state_cache_sizes,),
                    (upload_bandwidths, download_bandwidths, timestamps),
                    (block_number, block_hash, block_time, block_timestamp, propagation_time),
                    location,
                    startup_time,
                ) = serde_json::from_str(raw_value.get())?;
                FeedMessage::AddedNode {
                    node_id,
                    node: Box::new(NodeDetails {
                        name,
                        implementation,
                        version,
                        stash_address,
                        network_id,
                    }),
                    stats: NodeStats {
                        peer_count,
                        queued_tx_count,
                    },
                    io: NodeIO(state_cache_sizes),
                    hardware: Box::new(NodeHardware(
                        upload_bandwidths,
                        download_bandwidths,
                        timestamps,
                    )),
                    block_details: BlockDetails {
                        block_number,
                        block_hash,
                        block_time,
                        block_timestamp,
                        propagation_time,
                    },
                    location,
                    startup_time,
                }
            }
            // A catchall for messages we don't know/care about yet:
            _ => {
                let value = raw_value.to_string();
                FeedMessage::UnknownValue { action, value }
            }
        };
        Ok(feed_message)
    }
}

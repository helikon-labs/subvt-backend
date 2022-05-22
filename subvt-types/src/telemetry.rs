//! All Telemetry-related data types.
use anyhow::Context;
use serde::Deserialize;
use serde_json::value::RawValue;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct NodeStats {
    pub peer_count: u64,
    pub queued_tx_count: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeIO(
    // state cache size
    Vec<f32>,
);

#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeHardware(
    // upload bandwidth
    pub Vec<f64>,
    // download bandwidth
    pub Vec<f64>,
    // timestamps
    pub Vec<f64>,
);

#[derive(Debug, Deserialize, PartialEq)]
pub struct NodeLocation(
    // latitude
    pub f32,
    // longitude
    pub f32,
    // city
    pub String,
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeDetails {
    pub name: String,
    pub implementation: String,
    pub version: String,
    pub controller_address: Option<String>,
    pub network_id: Option<String>,
}

#[derive(Deserialize)]
pub struct Block {
    _block_hash: String,
    _block_height: u64,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct BlockDetails {
    pub block_number: u64,
    pub block_hash: String,
    pub block_time: u64,
    pub block_timestamp: u64,
    pub propagation_time: Option<u64>,
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
        node_id: u64,
        node_details: Box<NodeDetails>,
        stats: NodeStats,
        io: NodeIO,                  // can't losslessly deserialize
        hardware: Box<NodeHardware>, // can't losslessly deserialize
        block_details: BlockDetails,
        location: Option<NodeLocation>,
        startup_time: Option<u64>,
    },
    RemovedNode {
        node_id: u64,
    },
    LocatedNode {
        node_id: u64,
        latitude: f32,
        longitude: f32,
        city: String,
    },
    NodeImportedBlock {
        node_id: u64,
        block_details: BlockDetails,
    },
    NodeFinalizedBlock {
        node_id: u64,
        block_number: u64,
        block_hash: String,
    },
    NodeStatsUpdate {
        node_id: u64,
        stats: NodeStats,
    },
    NodeHardware {
        node_id: u64,
        hardware: NodeHardware,
    },
    TimeSync {
        time: u64,
    },
    AddedChain {
        name: String,
        genesis_hash: String,
        node_count: usize,
    },
    RemovedChain {
        genesis_hash: String,
    },
    SubscribedTo {
        genesis_hash: String,
    },
    UnsubscribedFrom {
        genesis_hash: String,
    },
    Pong {
        message: String,
    },
    AfgFinalized {
        address: String,
        block_number: u64,
        block_hash: String,
    },
    AfgReceivedPrevote {
        address: String,
        block_number: u64,
        block_hash: String,
        voter: Option<String>,
    },
    AfgReceivedPrecommit {
        address: String,
        block_number: u64,
        block_hash: String,
        voter: Option<String>,
    },
    AfgAuthoritySet {
        // Not used currently; not sure what "address" params are:
        a1: String,
        a2: String,
        a3: String,
        block_number: u64,
        block_hash: String,
    },
    StaleNode {
        node_id: u64,
    },
    NodeIOUpdate {
        node_id: u64,
        io: NodeIO,
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
                    (name, implementation, version, controller_address, network_id),
                    (peer_count, queued_tx_count),
                    (state_cache_sizes,),
                    (upload_bandwidths, download_bandwidths, timestamps),
                    (block_number, block_hash, block_time, block_timestamp, propagation_time),
                    location,
                    startup_time,
                ) = serde_json::from_str(raw_value.get())?;
                FeedMessage::AddedNode {
                    node_id,
                    node_details: Box::new(NodeDetails {
                        name,
                        implementation,
                        version,
                        controller_address,
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
            // removed node
            4 => {
                let node_id = serde_json::from_str(raw_value.get())?;
                FeedMessage::RemovedNode { node_id }
            }
            // located node
            5 => {
                let (node_id, latitude, longitude, city) = serde_json::from_str(raw_value.get())?;
                FeedMessage::LocatedNode {
                    node_id,
                    latitude,
                    longitude,
                    city,
                }
            }
            // node imported block
            6 => {
                let (node_id, block_details) = serde_json::from_str(raw_value.get())?;
                FeedMessage::NodeImportedBlock {
                    node_id,
                    block_details,
                }
            }
            // node finalized block
            7 => {
                let (node_id, block_number, block_hash) = serde_json::from_str(raw_value.get())?;
                FeedMessage::NodeFinalizedBlock {
                    node_id,
                    block_number,
                    block_hash,
                }
            }
            // node stats update
            8 => {
                let (node_id, stats) = serde_json::from_str(raw_value.get())?;
                FeedMessage::NodeStatsUpdate { node_id, stats }
            }
            // node hardware
            9 => {
                let (node_id, (upload_bandwidths, download_bandwidths, timestamps)) =
                    serde_json::from_str(raw_value.get())?;
                FeedMessage::NodeHardware {
                    node_id,
                    hardware: NodeHardware(upload_bandwidths, download_bandwidths, timestamps),
                }
            }
            // time sync
            10 => {
                let time = serde_json::from_str(raw_value.get())?;
                FeedMessage::TimeSync { time }
            }
            // added chain
            11 => {
                let (name, genesis_hash, node_count) = serde_json::from_str(raw_value.get())?;
                FeedMessage::AddedChain {
                    name,
                    genesis_hash,
                    node_count,
                }
            }
            // removed chain
            12 => {
                let genesis_hash = serde_json::from_str(raw_value.get())?;
                FeedMessage::RemovedChain { genesis_hash }
            }
            // subscribed to
            13 => {
                let genesis_hash = serde_json::from_str(raw_value.get())?;
                FeedMessage::SubscribedTo { genesis_hash }
            }
            // unsubscribed from
            14 => {
                let genesis_hash = serde_json::from_str(raw_value.get())?;
                FeedMessage::UnsubscribedFrom { genesis_hash }
            }
            // Pong
            15 => {
                let message = serde_json::from_str(raw_value.get())?;
                FeedMessage::Pong { message }
            }
            // afg finalized
            16 => {
                let (address, block_number, block_hash) = serde_json::from_str(raw_value.get())?;
                FeedMessage::AfgFinalized {
                    address,
                    block_number,
                    block_hash,
                }
            }
            // afg received prevote
            17 => {
                let (address, block_number, block_hash, voter) =
                    serde_json::from_str(raw_value.get())?;
                FeedMessage::AfgReceivedPrevote {
                    address,
                    block_number,
                    block_hash,
                    voter,
                }
            }
            // afg received precommit
            18 => {
                let (address, block_number, block_hash, voter) =
                    serde_json::from_str(raw_value.get())?;
                FeedMessage::AfgReceivedPrecommit {
                    address,
                    block_number,
                    block_hash,
                    voter,
                }
            }
            // afg authority set
            19 => {
                let (a1, a2, a3, block_number, block_hash) = serde_json::from_str(raw_value.get())?;
                FeedMessage::AfgAuthoritySet {
                    a1,
                    a2,
                    a3,
                    block_number,
                    block_hash,
                }
            }
            // stale node
            20 => {
                let node_id = serde_json::from_str(raw_value.get())?;
                FeedMessage::StaleNode { node_id }
            }
            // node io
            21 => {
                let (node_id, (state_cache_sizes,)) = serde_json::from_str(raw_value.get())?;
                FeedMessage::NodeIOUpdate {
                    node_id,
                    io: NodeIO(state_cache_sizes),
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

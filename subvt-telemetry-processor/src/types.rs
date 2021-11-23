use serde::Deserialize;

pub type BlockNumber = u64;
pub type Timestamp = u64;
pub type BlockHash = String;
pub type FeedNodeId = usize;
pub type MeanList<T> = [T; 20];

#[derive(Deserialize)]
pub struct NodeDetails(
    // name
    Box<str>,
    // implementation
    Box<str>,
    // version
    Box<str>,
    // validator
    Option<Box<str>>,
    //network_id
    Option<Box<str>>,
    // startup_time
    Option<Box<str>>,
);

#[derive(Deserialize)]
pub struct NodeStats(
    // peers
    u64,
    // txcount
    u64,
);

#[derive(Deserialize)]
pub struct NodeIO(
    // used_state_cache_size
    MeanList<f32>,
);

#[derive(Deserialize)]
pub struct Block(
    // hash
    BlockHash,
    // height
    BlockNumber,
);

#[derive(Deserialize)]
pub struct NodeHardware(
    // upload
    MeanList<f64>,
    // download
    MeanList<f64>,
    // chart_stamps
    MeanList<f64>,
);

#[derive(Deserialize)]
pub struct BlockDetails(
    BlockNumber,
    BlockHash,
    // block_time
    u64,
    // block_timestamp
    u64,
    // propagation_time
    Option<u64>,
);

#[derive(Deserialize)]
pub struct NodeLocation(
    // latitude
    f32,
    // longitude
    f32,
    // city
    Box<str>,
);

#[derive(Deserialize)]
pub struct Node(
    NodeDetails,
    NodeStats,
    NodeIO,
    NodeHardware,
    BlockDetails, // best
    Block,        // finalized
    u64,          // throttle
    Option<NodeLocation>,
    bool,              // stale
    Option<Timestamp>, // startup time
);

#[derive(Deserialize)]
pub struct AddedNode(pub FeedNodeId, pub Node);

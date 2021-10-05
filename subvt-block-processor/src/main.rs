//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_block_processor::BlockProcessor;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: BlockProcessor = BlockProcessor::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

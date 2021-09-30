//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_block_indexer::BlockIndexer;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: BlockIndexer = BlockIndexer::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

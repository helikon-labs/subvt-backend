//! Indexes historical block data to the PostreSQL database instance.

use async_trait::async_trait;
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_service_common::Service;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct BlockIndexer;

impl BlockIndexer {

}

#[async_trait]
impl Service for BlockIndexer {
    async fn run(&'static self) -> anyhow::Result<()> {
        Ok(())
    }
}
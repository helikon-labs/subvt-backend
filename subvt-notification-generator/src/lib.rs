//! Subscribes to the inactive validator list data on Redis and publishes the data
//! through WebSocket pub/sub.

use async_trait::async_trait;
use lazy_static::lazy_static;
use subvt_config::Config;
use subvt_service_common::Service;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NotificationGenerator;

#[async_trait(?Send)]
impl Service for NotificationGenerator {
    async fn run(&'static self) -> anyhow::Result<()> {
        Ok(())
    }
}

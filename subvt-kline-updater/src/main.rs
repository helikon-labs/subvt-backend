//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_kline_updater::KLineUpdater;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: KLineUpdater = KLineUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

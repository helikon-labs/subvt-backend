//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_network_status_updater::NetworkStatusUpdater;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: NetworkStatusUpdater = NetworkStatusUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

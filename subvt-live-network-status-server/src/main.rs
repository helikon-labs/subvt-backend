//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_live_network_status_server::LiveNetworkStatusPresenter;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: LiveNetworkStatusPresenter = LiveNetworkStatusPresenter::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

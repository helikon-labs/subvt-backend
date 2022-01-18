//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_network_status_server::NetworkStatusServer;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: NetworkStatusServer = NetworkStatusServer::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

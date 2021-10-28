//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_onekv_updater::OneKVUpdater;

lazy_static! {
    static ref SERVICE: OneKVUpdater = OneKVUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

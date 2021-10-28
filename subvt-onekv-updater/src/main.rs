//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_onekv_updater::OneKVUpdater;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: OneKVUpdater = OneKVUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

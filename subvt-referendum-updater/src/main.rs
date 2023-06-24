//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_referendum_updater::ReferendumUpdater;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: ReferendumUpdater = ReferendumUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_validator_list_updater::ValidatorListUpdater;

lazy_static! {
    static ref SERVICE: ValidatorListUpdater = ValidatorListUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

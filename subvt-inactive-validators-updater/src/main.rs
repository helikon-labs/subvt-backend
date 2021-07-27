//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_inactive_validators_updater::InactiveValidatorListUpdater;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: InactiveValidatorListUpdater = InactiveValidatorListUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

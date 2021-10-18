//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_thousand_validators_updater::ThousandValidatorsUpdater;

lazy_static! {
    static ref SERVICE: ThousandValidatorsUpdater = ThousandValidatorsUpdater::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

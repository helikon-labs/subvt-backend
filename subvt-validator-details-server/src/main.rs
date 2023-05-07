//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_validator_details_server::ValidatorDetailsServer;

lazy_static! {
    static ref SERVICE: ValidatorDetailsServer = ValidatorDetailsServer;
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

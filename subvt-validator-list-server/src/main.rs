//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_validator_list_server::ValidatorListServer;

lazy_static! {
    static ref SERVICE: ValidatorListServer = ValidatorListServer;
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_active_validator_list_server::ActiveValidatorListServer;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: ActiveValidatorListServer = ActiveValidatorListServer::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

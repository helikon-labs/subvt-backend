//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_inactive_validator_list_server::InactiveValidatorListPresenter;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: InactiveValidatorListPresenter = InactiveValidatorListPresenter::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_app_services::AppServices;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: AppServices = AppServices::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_app_service::AppService;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: AppService = AppService::default();
}

#[actix_web::main]
async fn main() {
    SERVICE.start().await;
}

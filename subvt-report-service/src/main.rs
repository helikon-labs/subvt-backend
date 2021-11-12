//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_report_service::ReportService;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: ReportService = ReportService::default();
}

#[actix_web::main]
async fn main() {
    SERVICE.start().await;
}

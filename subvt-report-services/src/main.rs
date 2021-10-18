//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_report_services::ReportServices;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: ReportServices = ReportServices::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

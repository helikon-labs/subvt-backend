//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_telemetry_processor::TelemetryProcessor;

lazy_static! {
    static ref SERVICE: TelemetryProcessor = TelemetryProcessor::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

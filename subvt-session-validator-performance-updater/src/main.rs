use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_session_validator_performance_updater::SessionValidatorPerformanceUpdater;

lazy_static! {
    static ref SERVICE: SessionValidatorPerformanceUpdater = SessionValidatorPerformanceUpdater;
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

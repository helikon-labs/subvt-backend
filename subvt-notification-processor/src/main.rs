//! See `./lib.rs` for details.

use once_cell::sync::OnceCell;
use subvt_notification_processor::NotificationProcessor;
use subvt_service_common::Service;

static SERVICE: OnceCell<NotificationProcessor> = OnceCell::new();

#[tokio::main]
async fn main() {
    let _ = SERVICE.set(NotificationProcessor::new().await.unwrap());
    SERVICE.get().unwrap().start().await;
}

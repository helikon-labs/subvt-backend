//! See `./lib.rs` for details.

use once_cell::sync::OnceCell;
use subvt_notification_generator::NotificationGenerator;
use subvt_service_common::Service;

static SERVICE: OnceCell<NotificationGenerator> = OnceCell::new();

#[tokio::main]
async fn main() {
    let _ = SERVICE.set(NotificationGenerator::new().await.unwrap());
    SERVICE.get().unwrap().start().await;
}

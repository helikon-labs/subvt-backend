//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_notification_generator::NotificationGenerator;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: NotificationGenerator = NotificationGenerator;
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

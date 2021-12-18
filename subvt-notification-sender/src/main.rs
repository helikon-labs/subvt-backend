//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_notification_sender::NotificationSender;
use subvt_service_common::Service;

lazy_static! {
    static ref SERVICE: NotificationSender = NotificationSender::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

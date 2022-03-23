use async_trait::async_trait;
use subvt_types::app::{Notification, NotificationChannel};

pub mod apns;
pub mod email;
pub mod fcm;
pub mod telegram;

#[derive(thiserror::Error, Clone, Debug)]
pub(crate) enum NotificationSenderError {
    #[error("Notification sender error: {0}")]
    Error(String),
}

#[async_trait]
pub(crate) trait NotificationSender: Sync + Send {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String>;
    async fn send_grouped(
        &self,
        network_id: u32,
        notification_type_code: &str,
        channel: &NotificationChannel,
        target: &str,
        notifications: &[Notification],
    ) -> anyhow::Result<String>;
}

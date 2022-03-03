use async_trait::async_trait;
use subvt_types::app::Notification;

pub mod apns;
pub mod email;
pub mod fcm;
pub mod telegram;

#[derive(thiserror::Error, Clone, Debug)]
pub(crate) enum NotificationSenderError {
    #[error("Notification sender error: {0}")]
    Error(String),
}

#[async_trait(?Send)]
pub(crate) trait NotificationSender: Sync + Send {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String>;
}

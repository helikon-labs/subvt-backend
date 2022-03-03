use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender, CONFIG};
use async_trait::async_trait;
use fcm::Client as FCMClient;
use log::{error, info};
use serde::Serialize;
use subvt_types::app::Notification;

#[derive(Serialize)]
struct FCMMessage {
    message: String,
}

pub(crate) struct FCMSender {
    fcm_client: FCMClient,
    content_provider: ContentProvider,
}

impl FCMSender {
    pub fn new() -> anyhow::Result<FCMSender> {
        let content_provider = ContentProvider::new(&CONFIG.notification_sender.template_dir_path)?;
        Ok(FCMSender {
            fcm_client: FCMClient::new(),
            content_provider,
        })
    }
}

#[async_trait(?Send)]
impl NotificationSender for FCMSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let message = FCMMessage {
            message: self
                .content_provider
                .get_push_notification_content(notification)?,
        };
        let mut builder = fcm::MessageBuilder::new(
            &CONFIG.notification_sender.fcm_api_key,
            &notification.notification_target,
        );
        builder.data(&message)?;
        match self.fcm_client.send(builder.finalize()).await {
            Ok(response) => {
                info!(
                    "FCM message sent succesfully for notification #{}.",
                    notification.id
                );
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                error!(
                    "FCM message send error for notification #{}: {:?}.",
                    notification.id, error,
                );
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
    }
}

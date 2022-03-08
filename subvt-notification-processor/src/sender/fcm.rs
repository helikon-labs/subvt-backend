use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender, CONFIG};
use async_trait::async_trait;
use fcm::Client as FCMClient;
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
    pub async fn new(content_provider: ContentProvider) -> anyhow::Result<FCMSender> {
        Ok(FCMSender {
            fcm_client: FCMClient::new(),
            content_provider,
        })
    }
}

#[async_trait]
impl NotificationSender for FCMSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let message = FCMMessage {
            message: self
                .content_provider
                .get_notification_content(notification)?
                .body_text
                .unwrap_or_else(|| {
                    panic!(
                        "Cannot get text content for FCM {} notification.",
                        notification.notification_type_code
                    )
                }),
        };
        let mut builder = fcm::MessageBuilder::new(
            &CONFIG.notification_processor.fcm_api_key,
            &notification.notification_target,
        );
        builder.data(&message)?;
        match self.fcm_client.send(builder.finalize()).await {
            Ok(response) => {
                log::info!(
                    "FCM message sent succesfully for notification #{}.",
                    notification.id
                );
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                log::error!(
                    "FCM message send error for notification #{}: {:?}.",
                    notification.id,
                    error,
                );
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
    }
}

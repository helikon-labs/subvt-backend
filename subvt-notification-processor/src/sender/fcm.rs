//! Firebase Cloud Messaging (FCM) sender. Sends notifications to Android devices.
use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender, CONFIG};
use async_trait::async_trait;
use fcm::Client as FCMClient;
use serde::Serialize;
use subvt_types::app::notification::{Notification, NotificationChannel};

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

impl FCMSender {
    async fn send_inner(&self, message: FCMMessage, target: &str) -> anyhow::Result<String> {
        let mut builder =
            fcm::MessageBuilder::new(&CONFIG.notification_processor.fcm_api_key, target);
        builder.data(&message)?;
        match self.fcm_client.send(builder.finalize()).await {
            Ok(response) => {
                log::info!("FCM message sent succesfully.");
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                log::error!("FCM message send error: {:?}.", error,);
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
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
        self.send_inner(message, &notification.notification_target)
            .await
    }

    async fn send_grouped(
        &self,
        network_id: u32,
        notification_type_code: &str,
        channel: &NotificationChannel,
        target: &str,
        notifications: &[Notification],
    ) -> anyhow::Result<String> {
        let message = FCMMessage {
            message: self
                .content_provider
                .get_grouped_notification_content(
                    network_id,
                    notification_type_code,
                    channel,
                    notifications,
                )?
                .body_text
                .unwrap_or_else(|| {
                    panic!(
                        "Cannot get text content for grouped FCM {} notification.",
                        notification_type_code
                    )
                }),
        };
        self.send_inner(message, target).await
    }
}

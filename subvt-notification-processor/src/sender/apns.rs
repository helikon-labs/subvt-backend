//! Apple Push Notification Service (APNS) sender. Sends notifications to Apple devices.
use crate::sender::{NotificationSender, NotificationSenderError};
use crate::{ContentProvider, CONFIG};
use a2::NotificationBuilder;
use async_trait::async_trait;
use subvt_types::app::{Notification, NotificationChannel};

pub(crate) struct APNSSender {
    apns_client: a2::Client,
    content_provider: ContentProvider,
}

impl APNSSender {
    pub async fn new(content_provider: ContentProvider) -> anyhow::Result<APNSSender> {
        let mut apns_key = std::fs::File::open(&CONFIG.notification_processor.apns_key_path)?;
        let apns_client = a2::Client::token(
            &mut apns_key,
            &CONFIG.notification_processor.apns_key_id,
            &CONFIG.notification_processor.apns_team_id,
            if CONFIG.notification_processor.apns_is_production {
                a2::Endpoint::Production
            } else {
                a2::Endpoint::Sandbox
            },
        )?;
        Ok(APNSSender {
            apns_client,
            content_provider,
        })
    }
}

impl APNSSender {
    async fn send_inner(&self, message: &str, target: &str) -> anyhow::Result<String> {
        let mut builder = a2::PlainNotificationBuilder::new(message);
        builder.set_sound("default");
        // builder.set_badge(1u32);
        let payload = builder.build(
            target,
            a2::NotificationOptions {
                apns_topic: Some(CONFIG.notification_processor.apns_topic.as_ref()),
                ..Default::default()
            },
        );
        match self.apns_client.send(payload).await {
            Ok(response) => {
                log::info!("APNS notification sent succesfully.");
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                log::error!("APNS notification send error: {:?}.", error,);
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
    }
}

#[async_trait]
impl NotificationSender for APNSSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let message = self
            .content_provider
            .get_notification_content(notification)?
            .body_text
            .unwrap_or_else(|| {
                panic!(
                    "Cannot get text content for APNS {} notification.",
                    notification.notification_type_code
                )
            });
        self.send_inner(&message, &notification.notification_target)
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
        let message = self
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
                    "Cannot get text content for APNS {} notification.",
                    notification_type_code
                )
            });
        self.send_inner(&message, target).await
    }
}

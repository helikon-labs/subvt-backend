use crate::sender::{NotificationSender, NotificationSenderError};
use crate::{ContentProvider, CONFIG};
use a2::NotificationBuilder;
use async_trait::async_trait;
use log::{error, info};
use subvt_types::app::Notification;

pub(crate) struct APNSSender {
    apns_client: a2::Client,
    content_provider: ContentProvider,
}

impl APNSSender {
    pub async fn new() -> anyhow::Result<APNSSender> {
        let mut apns_key = std::fs::File::open(&CONFIG.notification_processor.apns_key_location)?;
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
        let content_provider = ContentProvider::new().await?;
        Ok(APNSSender {
            apns_client,
            content_provider,
        })
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
        let mut builder = a2::PlainNotificationBuilder::new(&message);
        builder.set_sound("default");
        // builder.set_badge(1u32);
        let payload = builder.build(
            &notification.notification_target,
            a2::NotificationOptions {
                apns_topic: Some(CONFIG.notification_processor.apns_topic.as_ref()),
                ..Default::default()
            },
        );
        match self.apns_client.send(payload).await {
            Ok(response) => {
                info!("APNS notification {} sent succesfully.", notification.id);
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                error!(
                    "APNS notification {} send error: {:?}.",
                    notification.id, error,
                );
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
    }
}

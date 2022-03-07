use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender, CONFIG};
use async_trait::async_trait;
use frankenstein::{AsyncApi as TelegramClient, AsyncTelegramApi, ChatId, SendMessageParams};
use log::{error, info};
use subvt_types::app::Notification;

pub(crate) struct TelegramSender {
    telegram_client: TelegramClient,
    content_provider: ContentProvider,
}

impl TelegramSender {
    pub async fn new() -> anyhow::Result<TelegramSender> {
        let telegram_client = TelegramClient::new(&CONFIG.notification_processor.telegram_token);
        let content_provider = ContentProvider::new().await?;
        Ok(TelegramSender {
            telegram_client,
            content_provider,
        })
    }
}

#[async_trait]
impl NotificationSender for TelegramSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let message = self
            .content_provider
            .get_notification_content(notification)?
            .body_html
            .unwrap_or_else(|| {
                panic!(
                    "Cannot get HTML content for Telegram {} notification.",
                    notification.notification_type_code
                )
            });
        let params = SendMessageParams {
            chat_id: ChatId::Integer(notification.notification_target.parse()?),
            text: message,
            parse_mode: Some("html".to_string()),
            entities: None,
            disable_web_page_preview: Some(true),
            disable_notification: None,
            protect_content: None,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            reply_markup: None,
        };
        match self.telegram_client.send_message(&params).await {
            Ok(response) => {
                info!(
                    "Telegram notification sent succesfully for notification #{}.",
                    notification.id
                );
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                error!(
                    "Telegram notification send error for notification #{}: {:?}.",
                    notification.id, error,
                );
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
    }
}

use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender, CONFIG};
use async_trait::async_trait;
use subvt_telegram_bot::{
    api::AsyncApi as TelegramClient, AsyncTelegramApi, ChatId, ParseMode, SendMessageParams,
};
use subvt_types::app::{Notification, NotificationChannel};

pub(crate) struct TelegramSender {
    telegram_client: TelegramClient,
    content_provider: ContentProvider,
}

impl TelegramSender {
    pub async fn new(content_provider: ContentProvider) -> anyhow::Result<TelegramSender> {
        let telegram_client = TelegramClient::new(&CONFIG.notification_processor.telegram_token);
        Ok(TelegramSender {
            telegram_client,
            content_provider,
        })
    }
}

impl TelegramSender {
    async fn send_inner(&self, chat_id: ChatId, message: String) -> anyhow::Result<String> {
        let params = SendMessageParams {
            chat_id,
            text: message,
            parse_mode: Some(ParseMode::Html),
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
                log::info!("Telegram notification sent succesfully.");
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                log::error!("Telegram notification send error: {:?}.", error,);
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
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
        let chat_id = ChatId::Integer(notification.notification_target.parse()?);
        self.send_inner(chat_id, message).await
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
            .body_html
            .unwrap_or_else(|| {
                panic!(
                    "Cannot get grouped HTML content for Telegram {} notification.",
                    notification_type_code,
                )
            });
        let chat_id = ChatId::Integer(target.parse()?);
        self.send_inner(chat_id, message).await
    }
}

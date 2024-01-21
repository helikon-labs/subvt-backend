//! Telegram notification sender. Sends notification as messages to the SubVT Telegram Bot chats.
use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender};
use async_trait::async_trait;
use subvt_telegram_bot::{AsyncApi, AsyncTelegramApi, ChatId, ParseMode, SendMessageParams};
use subvt_types::app::notification::{Notification, NotificationChannel};

pub(crate) struct TelegramSender {
    telegram_client: AsyncApi,
    content_provider: ContentProvider,
}

impl TelegramSender {
    pub async fn new(
        api_token: &str,
        content_provider: ContentProvider,
    ) -> anyhow::Result<TelegramSender> {
        let telegram_client = AsyncApi::new(api_token);
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
            link_preview_options: None,
            disable_notification: None,
            protect_content: None,
            reply_parameters: None,
            reply_markup: None,
            message_thread_id: None,
        };
        match self.telegram_client.send_message(&params).await {
            Ok(response) => {
                log::info!("Telegram notification sent succesfully.");
                Ok(format!("{response:?}"))
            }
            Err(error) => {
                log::error!("Telegram notification send error: {:?}.", error,);
                Err(NotificationSenderError::Error(format!("{error:?}")).into())
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
                    "Cannot get grouped HTML content for Telegram {notification_type_code} notification.",
                )
            });
        let chat_id = ChatId::Integer(target.parse()?);
        self.send_inner(chat_id, message).await
    }
}

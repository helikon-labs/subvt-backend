use crate::ContentProvider;
use frankenstein::{AsyncApi, AsyncTelegramApi, ChatId, SendMessageParams};
use log::{error, info};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::Notification;

pub(crate) async fn send_telegram_message(
    postgres: &Arc<PostgreSQLAppStorage>,
    telegram_api: &Arc<AsyncApi>,
    content_provider: &Arc<ContentProvider>,
    notification: &Notification,
) -> anyhow::Result<()> {
    let content = content_provider.get_telegram_content(notification)?;
    postgres
        .mark_notification_processing(notification.id)
        .await?;
    let params = SendMessageParams {
        chat_id: ChatId::Integer(notification.notification_target.parse()?),
        text: content,
        parse_mode: Some("html".to_string()),
        entities: None,
        disable_web_page_preview: Some(true),
        disable_notification: None,
        protect_content: None,
        reply_to_message_id: None,
        allow_sending_without_reply: None,
        reply_markup: None,
    };
    match telegram_api.send_message(&params).await {
        Ok(response) => {
            info!(
                "Telegram notification sent succesfully for notification #{}.",
                notification.id
            );
            postgres.mark_notification_sent(notification.id).await?;
            postgres
                .mark_notification_delivered(notification.id)
                .await?;
            postgres
                .set_notification_log(notification.id, format!("{:?}", response).as_ref())
                .await?;
        }
        Err(error) => {
            error!(
                "Telegram notification send error for notification #{}: {:?}.",
                notification.id, error,
            );
            postgres.mark_notification_failed(notification.id).await?;
            postgres
                .set_notification_log(notification.id, format!("{:?}", error).as_ref())
                .await?;
        }
    }
    Ok(())
}

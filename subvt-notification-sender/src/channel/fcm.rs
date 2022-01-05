//! Firebase Cloud Messaging (FCM) notification sending logic for Android.

use crate::ContentProvider;
use log::{debug, error};
use serde::Serialize;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::Notification;

#[derive(Serialize)]
struct FCMMessage {
    message: String,
}

pub(crate) async fn send_fcm_message(
    config: &Config,
    postgres: &Arc<PostgreSQLAppStorage>,
    fcm_client: &Arc<fcm::Client>,
    content_provider: &Arc<ContentProvider>,
    notification: &Notification,
) -> anyhow::Result<()> {
    let message = FCMMessage {
        message: content_provider
            .get_push_notification_content_for_notification(config, notification)?,
    };
    let mut builder = fcm::MessageBuilder::new(
        &config.notification_sender.fcm_api_key,
        &notification.notification_target,
    );
    builder.data(&message)?;
    match fcm_client.send(builder.finalize()).await {
        Ok(response) => {
            debug!(
                "FCM message sent succesfully for notification #{}.",
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
                "FCM message send error for notification #{}: {:?}.",
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

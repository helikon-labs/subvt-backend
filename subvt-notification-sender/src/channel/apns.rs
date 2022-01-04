use crate::ContentProvider;
use a2::NotificationBuilder;
use log::{debug, error};
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::Notification;

pub(crate) async fn send_apple_push_notification(
    config: &Config,
    postgres: &Arc<PostgreSQLAppStorage>,
    apns_client: &Arc<a2::Client>,
    content_provider: &Arc<ContentProvider>,
    notification: &Notification,
) -> anyhow::Result<()> {
    let message =
        content_provider.get_push_notification_content_for_notification(config, notification)?;
    let mut builder = a2::PlainNotificationBuilder::new(&message);
    builder.set_sound("default");
    // builder.set_badge(1u32);
    let payload = builder.build(
        &notification.notification_target,
        a2::NotificationOptions {
            apns_topic: Some(config.notification_sender.apns_topic.as_ref()),
            ..Default::default()
        },
    );
    postgres
        .mark_notification_processing(notification.id)
        .await?;
    match apns_client.send(payload).await {
        Ok(response) => {
            debug!(
                "Apple push notification sent succesfully for notification #{}.",
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
                "Apple push notification send error for notification #{}: {:?}.",
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

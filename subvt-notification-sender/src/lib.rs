//! Sends the persisted notifications to various channels (email, push, SMS, GSM, etc.).

use crate::channel::email;
use crate::channel::email::Mailer;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, error};
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::{Notification, NotificationPeriodType};

mod channel;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NotificationSender;

impl NotificationSender {
    async fn send_notification(
        postgres: Arc<PostgreSQLAppStorage>,
        mailer: Arc<Mailer>,
        notification: Notification,
    ) -> anyhow::Result<()> {
        debug!(
            "Send {} notification #{} for {}.",
            notification.notification_channel_code,
            notification.id,
            notification.validator_account_id.to_ss58_check()
        );
        if notification.id % 4 == 0 {
            match notification.notification_channel_code.as_ref() {
                "email" => {
                    channel::email::send_email(&CONFIG, &postgres, &mailer, &notification).await?;
                }
                _ => todo!(
                    "Channel not implemented yet: {}",
                    notification.notification_channel_code
                ),
            }
        }
        Ok(())
    }

    async fn start_immediate_notification_processor(
        postgres: &Arc<PostgreSQLAppStorage>,
        mailer: &Arc<Mailer>,
    ) {
        loop {
            match postgres
                .get_pending_notifications_by_period_type(&NotificationPeriodType::Immediate)
                .await
            {
                Ok(notifications) => {
                    debug!("Got {} pending notifications.", notifications.len());
                    let mut futures = Vec::new();
                    for notification in notifications {
                        futures.push(NotificationSender::send_notification(
                            postgres.clone(),
                            mailer.clone(),
                            notification,
                        ));
                    }
                    if let Err(error) =
                        futures::future::try_join_all(futures.into_iter().map(tokio::spawn)).await
                    {
                        error!(
                            "Error while processing immediate pending notifications: {:?}",
                            error
                        );
                    }
                }
                Err(error) => error!("Error while getting pending notifications: {:?}", error),
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(
                CONFIG.notification_sender.sleep_millis,
            ))
            .await;
        }
    }
}

#[async_trait(?Send)]
impl Service for NotificationSender {
    async fn run(&'static self) -> anyhow::Result<()> {
        let postgres =
            Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
        let mailer = Arc::new(email::new_mailer(&CONFIG)?);
        debug!("Reset pending and failed notifications.");
        postgres.reset_pending_and_failed_notifications().await?;
        tokio::join!(NotificationSender::start_immediate_notification_processor(
            &postgres, &mailer
        ),);
        Ok(())
    }
}

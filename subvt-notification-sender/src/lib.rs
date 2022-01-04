//! Sends the persisted notifications to various channels (email, push, SMS, GSM, etc.).

use crate::channel::email;
use crate::channel::email::Mailer;
use crate::content::ContentProvider;
use async_trait::async_trait;
use chrono::{Datelike, Timelike, Utc};
use lazy_static::lazy_static;
use log::{debug, error};
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::{Notification, NotificationPeriodType};
use tokio::runtime::Builder;

mod channel;
mod content;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NotificationSender;

impl NotificationSender {
    async fn send_notification(
        postgres: Arc<PostgreSQLAppStorage>,
        mailer: Arc<Mailer>,
        content_provider: Arc<ContentProvider>,
        notification: Notification,
    ) -> anyhow::Result<()> {
        debug!(
            "Send {} notification #{} for {}.",
            notification.notification_channel_code,
            notification.id,
            notification.validator_account_id.to_ss58_check()
        );
        match notification.notification_channel_code.as_ref() {
            "email" => {
                channel::email::send_email(
                    &CONFIG,
                    &postgres,
                    &mailer,
                    &content_provider,
                    &notification,
                )
                .await?;
            }
            _ => todo!(
                "Notification channel not implemented yet: {}",
                notification.notification_channel_code
            ),
        }
        Ok(())
    }

    async fn start_immediate_notification_processor(
        postgres: &Arc<PostgreSQLAppStorage>,
        mailer: &Arc<Mailer>,
        content_provider: &Arc<ContentProvider>,
    ) {
        loop {
            debug!("Check immediate notifications.");
            NotificationSender::process_notifications(
                postgres,
                mailer,
                content_provider,
                NotificationPeriodType::Immediate,
                0,
            )
            .await;
            tokio::time::sleep(tokio::time::Duration::from_millis(
                CONFIG.notification_sender.sleep_millis,
            ))
            .await;
        }
    }

    fn start_hourly_and_daily_notification_processor(
        postgres: Arc<PostgreSQLAppStorage>,
        mailer: Arc<Mailer>,
        content_provider: Arc<ContentProvider>,
    ) {
        let tokio_rt = Builder::new_current_thread().enable_all().build().unwrap();
        std::thread::spawn(move || {
            let mut scheduler = job_scheduler::JobScheduler::new();
            scheduler.add(job_scheduler::Job::new(
                "0 0/1 * * * *".parse().unwrap(),
                || {
                    println!("Check hourly notifications.");
                    tokio_rt.block_on(NotificationSender::process_notifications(
                        &postgres,
                        &mailer,
                        &content_provider,
                        NotificationPeriodType::Hour,
                        Utc::now().hour(),
                    ));
                },
            ));
            scheduler.add(job_scheduler::Job::new(
                "0 12 * * * *".parse().unwrap(),
                || {
                    println!("Check daily notifications.");
                    tokio_rt.block_on(NotificationSender::process_notifications(
                        &postgres,
                        &mailer,
                        &content_provider,
                        NotificationPeriodType::Day,
                        Utc::now().day(),
                    ));
                },
            ));
            loop {
                scheduler.tick();
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        });
    }

    async fn process_notifications(
        postgres: &Arc<PostgreSQLAppStorage>,
        mailer: &Arc<Mailer>,
        content_provider: &Arc<ContentProvider>,
        period_type: NotificationPeriodType,
        period: u32,
    ) {
        match postgres
            .get_pending_notifications_by_period_type(&period_type, period)
            .await
        {
            Ok(notifications) => {
                debug!(
                    "Got {} pending {} notifications.",
                    notifications.len(),
                    period_type
                );
                let mut futures = Vec::new();
                for notification in notifications {
                    futures.push(NotificationSender::send_notification(
                        postgres.clone(),
                        mailer.clone(),
                        content_provider.clone(),
                        notification,
                    ));
                }
                if let Err(error) =
                    futures::future::try_join_all(futures.into_iter().map(tokio::spawn)).await
                {
                    error!(
                        "Error while processing pending {}-{} notifications: {:?}",
                        period, period_type, error,
                    );
                }
            }
            Err(error) => error!(
                "Error while getting pending {}-{} notifications: {:?}",
                period, period_type, error
            ),
        }
    }
}

#[async_trait(?Send)]
impl Service for NotificationSender {
    async fn run(&'static self) -> anyhow::Result<()> {
        let postgres =
            Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
        let mailer = Arc::new(email::new_mailer(&CONFIG)?);
        let content_provider = Arc::new(ContentProvider::new()?);
        debug!("Reset pending and failed notifications.");
        postgres.reset_pending_and_failed_notifications().await?;
        NotificationSender::start_hourly_and_daily_notification_processor(
            postgres.clone(),
            mailer.clone(),
            content_provider.clone(),
        );
        /*
        // hourly
        let postgres_1 = postgres.clone();
        let mailer_1 = mailer.clone();
        let content_provider_1 = content_provider.clone();
        let _ = scheduler.add(
            Job::new_async("0 * * * * *", |_, _| Box::pin(async move {
                debug!("Run hourly check.");
                NotificationSender::process_notifications(
                    &postgres_1,
                    &mailer_1,
                    &content_provider_1,
                    NotificationPeriodType::Hour,
                    1
                ).await;
            })).unwrap()
        );
         */

        tokio::join!(NotificationSender::start_immediate_notification_processor(
            &postgres,
            &mailer,
            &content_provider
        ),);
        Ok(())
    }
}

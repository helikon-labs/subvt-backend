//! Sends the persisted notifications to various channels (email, APNS, FCM, SMS, GSM, Telegram).

use crate::channel::email;
use crate::channel::email::Mailer;
use crate::content::ContentProvider;
use anyhow::Context;
use async_trait::async_trait;
use chrono::{Datelike, Timelike, Utc};
use frankenstein::AsyncApi;
use lazy_static::lazy_static;
use log::{debug, error};
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::{Notification, NotificationPeriodType};
use subvt_types::subvt::NetworkStatus;
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
        apns_client: Arc<a2::Client>,
        fcm_client: Arc<fcm::Client>,
        telegram_api: Arc<AsyncApi>,
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
                channel::email::send_email(&postgres, &mailer, &content_provider, &notification)
                    .await?;
            }
            "apns" => {
                channel::apns::send_apple_push_notification(
                    &postgres,
                    &apns_client,
                    &content_provider,
                    &notification,
                )
                .await?;
            }
            "fcm" => {
                channel::fcm::send_fcm_message(
                    &postgres,
                    &fcm_client,
                    &content_provider,
                    &notification,
                )
                .await?;
            }
            "telegram" => {
                channel::telegram::send_telegram_message(
                    &postgres,
                    &telegram_api,
                    &content_provider,
                    &notification,
                )
                .await?
            }
            _ => todo!(
                "Notification channel not implemented yet: {}",
                notification.notification_channel_code
            ),
        }
        Ok(())
    }

    /// Checks and sends notifications that should be sent immediately.
    async fn start_immediate_notification_processor(
        postgres: &Arc<PostgreSQLAppStorage>,
        mailer: &Arc<Mailer>,
        apns_client: &Arc<a2::Client>,
        fcm_client: &Arc<fcm::Client>,
        telegram_api: &Arc<AsyncApi>,
        content_provider: &Arc<ContentProvider>,
    ) {
        loop {
            debug!("Check immediate notifications.");
            NotificationSender::process_notifications(
                postgres,
                mailer,
                apns_client,
                fcm_client,
                telegram_api,
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

    /// Runs two cron-like jobs to process hourly and daily notifications.
    fn start_hourly_and_daily_notification_processor(
        postgres: Arc<PostgreSQLAppStorage>,
        mailer: Arc<Mailer>,
        apns_client: Arc<a2::Client>,
        fcm_client: Arc<fcm::Client>,
        telegram_api: Arc<AsyncApi>,
        content_provider: Arc<ContentProvider>,
    ) -> anyhow::Result<()> {
        let tokio_rt = Builder::new_current_thread().enable_all().build()?;
        std::thread::spawn(move || {
            let mut scheduler = job_scheduler::JobScheduler::new();
            // hourly jobs
            scheduler.add(job_scheduler::Job::new(
                "0 0/1 * * * *".parse().unwrap(),
                || {
                    tokio_rt.block_on(NotificationSender::process_notifications(
                        &postgres,
                        &mailer,
                        &apns_client,
                        &fcm_client,
                        &telegram_api,
                        &content_provider,
                        NotificationPeriodType::Hour,
                        Utc::now().hour(),
                    ));
                },
            ));
            // daily jobs - send at midday UTC
            scheduler.add(job_scheduler::Job::new(
                "0 12 * * * *".parse().unwrap(),
                || {
                    println!("Check daily notifications.");
                    tokio_rt.block_on(NotificationSender::process_notifications(
                        &postgres,
                        &mailer,
                        &apns_client,
                        &fcm_client,
                        &telegram_api,
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
        Ok(())
    }

    /// Subscribes to the network status notifications from Redis (which are generated by
    /// `subvt-network-status-updater`) and processes epoch and era notification at epoch
    /// and era changes.
    fn start_era_and_epoch_notification_processor(
        postgres: Arc<PostgreSQLAppStorage>,
        mailer: Arc<Mailer>,
        apns_client: Arc<a2::Client>,
        fcm_client: Arc<fcm::Client>,
        telegram_api: Arc<AsyncApi>,
        content_provider: Arc<ContentProvider>,
    ) -> anyhow::Result<()> {
        let redis_client = redis::Client::open(CONFIG.redis.url.as_str()).context(format!(
            "Cannot connect to Redis at URL {}.",
            CONFIG.redis.url
        ))?;
        let mut data_connection = redis_client.get_connection()?;
        let mut pub_sub_connection = redis_client.get_connection()?;
        let tokio_rt = Builder::new_current_thread().enable_all().build()?;
        std::thread::spawn(move || {
            let mut active_era_index = 0;
            let mut current_epoch_index = 0;
            let mut pub_sub = pub_sub_connection.as_pubsub();
            pub_sub
                .subscribe(format!(
                    "subvt:{}:network_status:publish:best_block_number",
                    CONFIG.substrate.chain
                ))
                .unwrap();
            loop {
                let _ = pub_sub.get_message();
                let key = format!("subvt:{}:network_status", CONFIG.substrate.chain);
                let status_json_string: String = redis::cmd("GET")
                    .arg(key)
                    .query(&mut data_connection)
                    .unwrap();
                let status: NetworkStatus = serde_json::from_str(&status_json_string).unwrap();
                // process epoch notifications if epoch has changed
                if current_epoch_index != status.current_epoch.index {
                    tokio_rt.block_on(NotificationSender::process_notifications(
                        &postgres,
                        &mailer,
                        &apns_client,
                        &fcm_client,
                        &telegram_api,
                        &content_provider,
                        NotificationPeriodType::Epoch,
                        current_epoch_index as u32,
                    ));
                    current_epoch_index = status.current_epoch.index;
                }
                // process era notifications if epoch has changed
                if active_era_index != status.active_era.index {
                    tokio_rt.block_on(NotificationSender::process_notifications(
                        &postgres,
                        &mailer,
                        &apns_client,
                        &fcm_client,
                        &telegram_api,
                        &content_provider,
                        NotificationPeriodType::Era,
                        active_era_index,
                    ));
                    active_era_index = status.active_era.index;
                }
            }
        });
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_notifications(
        postgres: &Arc<PostgreSQLAppStorage>,
        mailer: &Arc<Mailer>,
        apns_client: &Arc<a2::Client>,
        fcm_client: &Arc<fcm::Client>,
        telegram_api: &Arc<AsyncApi>,
        content_provider: &Arc<ContentProvider>,
        period_type: NotificationPeriodType,
        period: u32,
    ) {
        debug!(
            "Process {} notification for period {}.",
            period_type, period,
        );
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
                for notification in notifications {
                    let postgres = postgres.clone();
                    let mailer = mailer.clone();
                    let apns_client = apns_client.clone();
                    let fcm_client = fcm_client.clone();
                    let telegram_api = telegram_api.clone();
                    let content_provider = content_provider.clone();
                    let period_type = period_type.clone();
                    tokio::spawn(async move {
                        if let Err(error) = NotificationSender::send_notification(
                            postgres.clone(),
                            mailer.clone(),
                            apns_client.clone(),
                            fcm_client.clone(),
                            telegram_api.clone(),
                            content_provider.clone(),
                            notification,
                        )
                        .await
                        {
                            error!(
                                "Error while processing pending {}-{} notifications: {:?}",
                                period, period_type, error,
                            );
                        }
                    });
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
        let mailer = Arc::new(email::new_mailer()?);
        let content_provider = Arc::new(ContentProvider::new(
            &CONFIG.notification_sender.template_dir_path,
        )?);
        let mut apns_key = std::fs::File::open(&CONFIG.notification_sender.apns_key_location)?;
        let apns_client = Arc::new(a2::Client::token(
            &mut apns_key,
            &CONFIG.notification_sender.apns_key_id,
            &CONFIG.notification_sender.apns_team_id,
            if CONFIG.notification_sender.apns_is_production {
                a2::Endpoint::Production
            } else {
                a2::Endpoint::Sandbox
            },
        )?);
        let fcm_client = Arc::new(fcm::Client::new());
        let telegram_api = Arc::new(AsyncApi::new(&CONFIG.notification_sender.telegram_token));
        debug!("Reset pending and failed notifications.");
        postgres.reset_pending_and_failed_notifications().await?;
        NotificationSender::start_era_and_epoch_notification_processor(
            postgres.clone(),
            mailer.clone(),
            apns_client.clone(),
            fcm_client.clone(),
            telegram_api.clone(),
            content_provider.clone(),
        )?;
        NotificationSender::start_hourly_and_daily_notification_processor(
            postgres.clone(),
            mailer.clone(),
            apns_client.clone(),
            fcm_client.clone(),
            telegram_api.clone(),
            content_provider.clone(),
        )?;
        tokio::join!(NotificationSender::start_immediate_notification_processor(
            &postgres,
            &mailer,
            &apns_client,
            &fcm_client,
            &telegram_api,
            &content_provider
        ),);
        Ok(())
    }
}

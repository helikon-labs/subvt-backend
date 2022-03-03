//! Sends the persisted notifications to various channels (email, APNS, FCM, SMS, GSM, Telegram).

use crate::content::ContentProvider;
use crate::sender::apns::APNSSender;
use crate::sender::email::EmailSender;
use crate::sender::fcm::FCMSender;
use crate::sender::telegram::TelegramSender;
use crate::sender::NotificationSender;
use anyhow::Context;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::info;
use redis::Client as RedisClient;
use std::collections::HashMap;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::NotificationChannel;

mod content;
mod processor;
mod sender;

type SenderMap = HashMap<NotificationChannel, Arc<Box<dyn NotificationSender>>>;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub struct NotificationProcessor {
    postgres: Arc<PostgreSQLAppStorage>,
    redis: RedisClient,
    senders: SenderMap,
}

impl NotificationProcessor {
    fn prepare_senders() -> anyhow::Result<SenderMap> {
        let mut senders = HashMap::new();
        senders.insert(
            NotificationChannel::APNS,
            Arc::new(Box::new(APNSSender::new()?) as Box<dyn NotificationSender>),
        );
        senders.insert(
            NotificationChannel::Email,
            Arc::new(Box::new(EmailSender::new()?) as Box<dyn NotificationSender>),
        );
        senders.insert(
            NotificationChannel::FCM,
            Arc::new(Box::new(FCMSender::new()?) as Box<dyn NotificationSender>),
        );
        senders.insert(
            NotificationChannel::Telegram,
            Arc::new(Box::new(TelegramSender::new()?) as Box<dyn NotificationSender>),
        );
        Ok(senders)
    }

    pub async fn new() -> anyhow::Result<NotificationProcessor> {
        let postgres =
            Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
        let redis = redis::Client::open(CONFIG.redis.url.as_str()).context(format!(
            "Cannot connect to Redis at URL {}.",
            CONFIG.redis.url
        ))?;
        Ok(NotificationProcessor {
            postgres,
            redis,
            senders: NotificationProcessor::prepare_senders()?,
        })
    }
}

#[async_trait(?Send)]
impl Service for NotificationProcessor {
    async fn run(&'static self) -> anyhow::Result<()> {
        info!("Reset pending and failed notifications.");
        self.postgres
            .reset_pending_and_failed_notifications()
            .await?;
        info!("Start notification processors.");
        tokio::try_join!(
            self.start_immediate_notification_processor(),
            self.start_hourly_and_daily_notification_processor(),
            self.start_era_and_epoch_notification_processor(),
        )?;
        Ok(())
    }
}

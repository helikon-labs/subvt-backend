//! Sends the persisted notifications to various channels (email, APNS, FCM, SMS, GSM, Telegram).
#![warn(clippy::disallowed_types)]
use crate::content::ContentProvider;
// use crate::sender::apns::APNSSender;
use crate::sender::email::EmailSender;
use crate::sender::fcm::FCMSender;
use crate::sender::telegram::TelegramSender;
use crate::sender::NotificationSender;
use async_trait::async_trait;
use itertools::Itertools;
use lazy_static::lazy_static;
use rustc_hash::FxHashMap as HashMap;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::{Network, NotificationChannel};

mod content;
pub(crate) mod metrics;
mod processor;
mod sender;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

/// Senders for different notification channels.
pub(crate) struct SenderRepository {
    // apns_sender: Arc<Box<dyn NotificationSender>>,
    email_sender: Arc<Box<dyn NotificationSender>>,
    fcm_sender: Arc<Box<dyn NotificationSender>>,
    kusama_telegram_sender: Arc<Box<dyn NotificationSender>>,
    polkadot_telegram_sender: Arc<Box<dyn NotificationSender>>,
}

impl SenderRepository {
    pub(crate) async fn new(
        network_map: &HashMap<u32, Network>,
    ) -> anyhow::Result<SenderRepository> {
        let content_provider = ContentProvider::new(network_map.clone())?;
        /*
        let apns_sender = Arc::new(Box::new(APNSSender::new(content_provider.clone()).await?)
            as Box<dyn NotificationSender>);
         */
        let email_sender = Arc::new(Box::new(EmailSender::new(content_provider.clone()).await?)
            as Box<dyn NotificationSender>);
        let fcm_sender = Arc::new(Box::new(FCMSender::new(content_provider.clone()).await?)
            as Box<dyn NotificationSender>);
        let kusama_telegram_sender = Arc::new(Box::new(
            TelegramSender::new(
                &CONFIG.notification_processor.kusama_telegram_api_token,
                content_provider.clone(),
            )
            .await?,
        ) as Box<dyn NotificationSender>);
        let polkadot_telegram_sender = Arc::new(Box::new(
            TelegramSender::new(
                &CONFIG.notification_processor.polkadot_telegram_api_token,
                content_provider.clone(),
            )
            .await?,
        ) as Box<dyn NotificationSender>);
        Ok(SenderRepository {
            // apns_sender,
            email_sender,
            fcm_sender,
            kusama_telegram_sender,
            polkadot_telegram_sender,
        })
    }

    /// Returns the sender for a specific notification channel.
    pub(crate) fn get_sender(
        &self,
        channel: &NotificationChannel,
        network_id: u32,
    ) -> Arc<Box<dyn NotificationSender>> {
        match channel {
            // NotificationChannel::APNS => self.apns_sender.clone(),
            NotificationChannel::Email => self.email_sender.clone(),
            NotificationChannel::FCM => self.fcm_sender.clone(),
            NotificationChannel::Telegram => match network_id {
                1 => self.kusama_telegram_sender.clone(),
                2 => self.polkadot_telegram_sender.clone(),
                _ => unimplemented!(
                    "Telegram sender not implemeneted for network with id {}",
                    network_id
                ),
            },
            NotificationChannel::APNS => unimplemented!("APNS sender not implemented."),
            NotificationChannel::SMS => unimplemented!("SMS sender not implemented."),
            NotificationChannel::GSM => unimplemented!("GSM sender not implemented."),
        }
    }
}

/// Notification processor, has the SubVT application database to access notifications,
/// a repository of notification senders for different channels, and a map of network ids to
/// supported networks.
pub struct NotificationProcessor {
    postgres: Arc<PostgreSQLAppStorage>,
    sender_repository: SenderRepository,
    network_map: HashMap<u32, Network>,
}

impl NotificationProcessor {
    async fn get_network_map(
        postgres: &PostgreSQLAppStorage,
    ) -> anyhow::Result<HashMap<u32, Network>> {
        let networks = postgres.get_networks().await?;
        let mut network_map = HashMap::default();
        for network in networks {
            network_map.insert(network.id, network.clone());
        }
        Ok(network_map)
    }

    pub async fn new() -> anyhow::Result<NotificationProcessor> {
        let postgres =
            Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
        let network_map = NotificationProcessor::get_network_map(&postgres).await?;
        Ok(NotificationProcessor {
            postgres,
            sender_repository: SenderRepository::new(&network_map).await?,
            network_map,
        })
    }
}

#[async_trait(?Send)]
impl Service for NotificationProcessor {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.notification_processor_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!("Reset pending notifications.");
        self.postgres.reset_pending_notifications().await?;
        log::info!("Start notification processors.");
        self.start_hourly_and_daily_notification_processor()?;
        let networks = self.network_map.values().collect_vec();
        for network in networks {
            let network = network.clone().to_owned();
            std::thread::spawn(move || loop {
                let tokio_rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                if let Err(error) =
                    tokio_rt.block_on(self.start_era_and_epoch_notification_processor(&network))
                {
                    log::error!(
                        "Error while starting era and epoch notification processor for {}:{:?}",
                        network.display,
                        error
                    );
                }
                log::error!(
                        "Era and epoch notification processor for {} is going to be restarted after {} seconds.",
                        network.display,
                        CONFIG.common.recovery_retry_seconds,
                    );
                std::thread::sleep(std::time::Duration::from_secs(
                    CONFIG.common.recovery_retry_seconds,
                ));
            });
        }
        self.start_immediate_notification_processor().await?;
        Ok(())
    }
}

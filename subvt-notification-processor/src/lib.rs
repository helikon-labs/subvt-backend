//! Sends the persisted notifications to various channels (email, APNS, FCM, SMS, GSM, Telegram).

use crate::content::ContentProvider;
use crate::sender::apns::APNSSender;
use crate::sender::email::EmailSender;
use crate::sender::fcm::FCMSender;
use crate::sender::telegram::TelegramSender;
use crate::sender::NotificationSender;
use async_trait::async_trait;
use itertools::Itertools;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::{Network, NotificationChannel};

mod content;
pub(crate) mod metrics;
mod processor;
mod sender;

type SenderMap = HashMap<NotificationChannel, Arc<Box<dyn NotificationSender>>>;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

pub struct NotificationProcessor {
    postgres: Arc<PostgreSQLAppStorage>,
    senders: SenderMap,
    network_map: HashMap<u32, Network>,
}

impl NotificationProcessor {
    async fn prepare_senders(network_map: &HashMap<u32, Network>) -> anyhow::Result<SenderMap> {
        let mut senders = HashMap::new();
        let content_provider = ContentProvider::new(network_map.clone())?;
        senders.insert(
            NotificationChannel::APNS,
            Arc::new(Box::new(APNSSender::new(content_provider.clone()).await?)
                as Box<dyn NotificationSender>),
        );
        senders.insert(
            NotificationChannel::Email,
            Arc::new(Box::new(EmailSender::new(content_provider.clone()).await?)
                as Box<dyn NotificationSender>),
        );
        senders.insert(
            NotificationChannel::FCM,
            Arc::new(Box::new(FCMSender::new(content_provider.clone()).await?)
                as Box<dyn NotificationSender>),
        );
        senders.insert(
            NotificationChannel::Telegram,
            Arc::new(
                Box::new(TelegramSender::new(content_provider.clone()).await?)
                    as Box<dyn NotificationSender>,
            ),
        );
        Ok(senders)
    }

    async fn get_network_map(
        postgres: &PostgreSQLAppStorage,
    ) -> anyhow::Result<HashMap<u32, Network>> {
        let networks = postgres.get_networks().await?;
        let mut network_map = HashMap::new();
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
            senders: NotificationProcessor::prepare_senders(&network_map).await?,
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
        log::info!("Reset pending and failed notifications.");
        self.postgres
            .reset_pending_and_failed_notifications()
            .await?;
        log::info!("Start notification processors.");
        self.start_hourly_and_daily_notification_processor()?;
        let networks = self.network_map.values().collect_vec();
        for network in networks {
            let network = network.clone().to_owned();
            std::thread::spawn(move || {
                let tokio_rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                let _ =
                    tokio_rt.block_on(self.start_era_and_epoch_notification_processor(&network));
            });
        }
        self.start_immediate_notification_processor().await?;
        Ok(())
    }
}

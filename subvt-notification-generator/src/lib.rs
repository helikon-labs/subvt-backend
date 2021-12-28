//! Subscribes to the inactive validator list data on Redis and publishes the data
//! through WebSocket pub/sub.
//!
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use serde::Serialize;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::{Notification, UserNotificationRule};
use subvt_types::crypto::AccountId;
use tokio::runtime::Builder;

mod processor;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NotificationGenerator;

impl NotificationGenerator {
    async fn generate_notifications<T: Clone + Serialize>(
        config: &Config,
        app_postgres: &PostgreSQLAppStorage,
        rules: &[UserNotificationRule],
        validator_account_id: &AccountId,
        notification_data: Option<&T>,
    ) -> anyhow::Result<()> {
        for rule in rules {
            debug!(
                "Generate {} notification for {}.",
                rule.notification_type.code,
                validator_account_id.to_ss58_check(),
            );
            for channel in &rule.notification_channels {
                let notification = Notification {
                    id: 0,
                    user_id: rule.user_id,
                    user_notification_rule_id: rule.id,
                    network_id: config.substrate.network_id,
                    period_type: rule.period_type.clone(),
                    period: rule.period,
                    validator_account_id: validator_account_id.clone(),
                    notification_type_code: rule.notification_type.code.clone(),
                    user_notification_channel_id: channel.id,
                    notification_channel_code: channel.channel_code.clone(),
                    notification_target: channel.target.clone(),
                    log: None,
                    created_at: None,
                    sent_at: None,
                    delivered_at: None,
                    read_at: None,
                    data: notification_data,
                };
                let _ = app_postgres.save_notification(&notification).await?;
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl Service for NotificationGenerator {
    async fn run(&'static self) -> anyhow::Result<()> {
        let tokio_rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            tokio_rt.block_on(NotificationGenerator::process_validator_list_updates(
                &CONFIG,
            ));
        });
        NotificationGenerator::start_processing_blocks(&CONFIG).await
    }
}

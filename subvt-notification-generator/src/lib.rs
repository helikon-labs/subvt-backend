//! Subscribes to the inactive validator list data on Redis and publishes the data
//! through WebSocket pub/sub.
//!
use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::Serialize;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_types::app::{Block, Notification, UserNotificationRule};
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
        maybe_block: &Option<&Block>,
        (extrinsic_index, event_index): (Option<u32>, Option<u32>),
        rules: &[UserNotificationRule],
        validator_account_id: &AccountId,
        (parameter_type_id, parameter_value, notification_data): (
            Option<u32>,
            Option<String>,
            Option<T>,
        ),
    ) -> anyhow::Result<()> {
        for rule in rules {
            println!(
                "Generate {} notification for {}.",
                rule.notification_type.code, validator_account_id,
            );
            let (block_hash, block_number, block_timestamp) = if let Some(block) = maybe_block {
                (
                    Some(block.hash.clone()),
                    Some(block.number),
                    block.timestamp,
                )
            } else {
                (None, None, None)
            };
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
                    parameter_type_id,
                    parameter_value: parameter_value.clone(),
                    block_hash: block_hash.clone(),
                    block_number,
                    block_timestamp,
                    extrinsic_index,
                    event_index,
                    user_notification_channel_id: channel.id,
                    notification_channel_code: channel.channel_code.clone(),
                    notification_target: channel.target.clone(),
                    log: None,
                    created_at: None,
                    sent_at: None,
                    delivered_at: None,
                    read_at: None,
                    data: notification_data.clone(),
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

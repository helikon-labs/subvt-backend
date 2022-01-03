//! Subscribes to the inactive validator list data on Redis and publishes the data
//! through WebSocket pub/sub.
//!
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::debug;
use serde::Serialize;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
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
        substrate_client: &Arc<SubstrateClient>,
        rules: &[UserNotificationRule],
        block_number: u64,
        validator_account_id: &AccountId,
        notification_data: Option<&T>,
    ) -> anyhow::Result<()> {
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let account_json = if let Some(account) = substrate_client
            .get_accounts(&[validator_account_id.clone()], &block_hash)
            .await?
            .get(0)
        {
            Some(serde_json::to_string(account)?)
        } else {
            None
        };
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
                    validator_account_json: account_json.clone(),
                    notification_type_code: rule.notification_type.code.clone(),
                    user_notification_channel_id: channel.id,
                    notification_channel_code: channel.channel_code.clone(),
                    notification_target: channel.target.clone(),
                    log: None,
                    created_at: None,
                    sent_at: None,
                    delivered_at: None,
                    read_at: None,
                    data_json: if let Ok(data_json) = serde_json::to_string(&notification_data) {
                        Some(data_json)
                    } else {
                        None
                    },
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
        let substrate_client = Arc::new(SubstrateClient::new(&CONFIG).await?);
        let tokio_rt = Builder::new_current_thread().enable_all().build().unwrap();
        let validator_list_processor_substrate_client = substrate_client.clone();
        // start processing validator list updates
        std::thread::spawn(move || {
            tokio_rt.block_on(NotificationGenerator::process_validator_list_updates(
                &CONFIG,
                validator_list_processor_substrate_client,
            ));
        });
        // start processing new blocks
        NotificationGenerator::start_processing_blocks(&CONFIG, substrate_client).await
    }
}

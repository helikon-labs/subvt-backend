//! Generates notifications according to the notification rules depending on three sources of data:
//! 1. Validator list updates from Redis, updated by `subvt-validator-list-updater`, and published
//! using the Redis notification (PUBLISH) support.
//! 2. Events and extrinsics in new blocks. Block are processed by `subvt-block-processor`, and the
//! finishing of the processing of a block is signalled by the processor by means of PostgreSQL
//! notifications.
//! 3. Regular Telemetry checks (this is work in progress still).
use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::Serialize;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::{Notification, UserNotificationRule};
use subvt_types::crypto::AccountId;

mod inspect;
mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NotificationGenerator;

impl NotificationGenerator {
    /// Persist notifications for a validator, which will later be be processed by
    /// `subvt-notification-sender`.
    async fn generate_notifications<T: Clone + Serialize>(
        &self,
        app_postgres: Arc<PostgreSQLAppStorage>,
        rules: &[UserNotificationRule],
        block_number: u64,
        maybe_validator_account_id: &Option<AccountId>,
        notification_data: Option<&T>,
    ) -> anyhow::Result<()> {
        if rules.is_empty() {
            return Ok(());
        }
        let substrate_client: Arc<SubstrateClient> = Arc::new(SubstrateClient::new(&CONFIG).await?);
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        // get account information for the validator stash address, which is used to display
        // identity information if exists
        let account_json = if let Some(validator_account_id) = maybe_validator_account_id.as_ref() {
            if let Some(account) = substrate_client
                .get_accounts(&[*validator_account_id], &block_hash)
                .await?
                .get(0)
            {
                Some(serde_json::to_string(account)?)
            } else {
                None
            }
        } else {
            None
        };
        // create separate notifications for each rule and notification channel
        for rule in rules {
            if let Some(validator_account_id) = maybe_validator_account_id {
                log::debug!(
                    "Generate {} notification for {:?}.",
                    rule.notification_type.code,
                    validator_account_id.to_ss58_check(),
                );
            } else {
                log::debug!("Generate {} notification.", rule.notification_type.code,);
            }
            for channel in &rule.notification_channels {
                let notification = Notification {
                    id: 0,
                    user_id: rule.user_id,
                    user_notification_rule_id: rule.id,
                    network_id: CONFIG.substrate.network_id,
                    period_type: rule.period_type,
                    period: rule.period,
                    validator_account_id: *maybe_validator_account_id,
                    validator_account_json: account_json.clone(),
                    notification_type_code: rule.notification_type.code.clone(),
                    user_notification_channel_id: channel.id,
                    notification_channel: channel.channel,
                    notification_target: channel.target.clone(),
                    error_log: None,
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
                metrics::notification_counter(
                    &rule.notification_type.code,
                    &format!("{}", channel.channel),
                )
                .inc();
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl Service for NotificationGenerator {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.notification_generator_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        tokio::spawn(self.start_block_inspection());
        self.start_validator_list_inspection().await?;
        Ok(())
    }
}

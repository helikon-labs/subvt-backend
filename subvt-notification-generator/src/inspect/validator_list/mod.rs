//! Checks validator changes for notifications. Validator list in Redis gets updated by
//! `subvt-validator-list-updater`, and the update is notified using the Redis PUBLISH function.
//! Keeps a copy of the validator list in heap memory (vector) to track changes.

use crate::{metrics, NotificationGenerator, CONFIG};
use anyhow::Context;
use futures_util::StreamExt as _;
use redis::aio::Connection as RedisConnection;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::subvt::ValidatorDetails;

mod add;
mod init;
mod onekv;
mod remove;
mod unclaimed_payout;
mod update;

impl NotificationGenerator {
    /// Called after each validator list update PUBLISH event.
    #[allow(clippy::too_many_arguments)]
    async fn inspect_validator_list_update(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        redis_connection: &mut RedisConnection,
        validator_map: &mut HashMap<String, ValidatorDetails>,
        finalized_block_number: u64,
        last_active_era_index: &AtomicU32,
    ) -> anyhow::Result<()> {
        log::info!(
            "Process new update from validator list updater. Block #{}.",
            finalized_block_number
        );
        let redis_storage_prefix = format!(
            "subvt:{}:validators:{}",
            CONFIG.substrate.chain, finalized_block_number
        );
        let active_validator_account_ids: HashSet<String> = redis::cmd("SMEMBERS")
            .arg(format!("{}:active:account_id_set", redis_storage_prefix))
            .query_async(redis_connection)
            .await?;
        let inactive_validator_account_ids: HashSet<String> = redis::cmd("SMEMBERS")
            .arg(format!("{}:inactive:account_id_set", redis_storage_prefix))
            .query_async(redis_connection)
            .await?;
        let all_validator_account_ids: HashSet<String> = active_validator_account_ids
            .union(&inactive_validator_account_ids)
            .cloned()
            .collect();
        if validator_map.is_empty() {
            log::info!("Validator map is empty. Populate.");
            init::populate_validator_map(
                redis_connection,
                &redis_storage_prefix,
                &active_validator_account_ids,
                &all_validator_account_ids,
                validator_map,
            )
            .await?;
            return Ok(());
        }
        let prev_validator_account_ids: HashSet<String> = validator_map.keys().cloned().collect();
        // new validators
        let added_validator_ids = &all_validator_account_ids - &prev_validator_account_ids;
        self.add_validators(
            network_postgres.clone(),
            redis_connection,
            &redis_storage_prefix,
            validator_map,
            finalized_block_number,
            &active_validator_account_ids,
            &added_validator_ids,
        )
        .await?;
        // removed validators
        let removed_validator_ids = &prev_validator_account_ids - &all_validator_account_ids;
        self.remove_validators(
            network_postgres.clone(),
            validator_map,
            finalized_block_number,
            &removed_validator_ids,
        )
        .await?;
        // remaining validators :: check for updates
        let remaining_validator_ids = &all_validator_account_ids - &added_validator_ids;
        log::debug!("Checking for changes in existing validators.");
        for validator_id in &remaining_validator_ids {
            let validator_prefix = format!(
                "{}:{}:validator:{}",
                redis_storage_prefix,
                if active_validator_account_ids.contains(validator_id) {
                    "active"
                } else {
                    "inactive"
                },
                validator_id
            );
            if let Some(updated) = self
                .inspect_validator_changes(
                    network_postgres.clone(),
                    app_postgres.clone(),
                    redis_connection,
                    &validator_prefix,
                    finalized_block_number,
                    validator_map.get(validator_id).unwrap(),
                )
                .await?
            {
                validator_map.insert(validator_id.clone(), updated);
            }
        }
        // unclaimed payouts
        self.inspect_unclaimed_payouts(
            network_postgres,
            app_postgres,
            redis_connection,
            &redis_storage_prefix,
            last_active_era_index,
            finalized_block_number,
            validator_map,
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn start_validator_list_inspection(&'static self) -> anyhow::Result<()> {
        loop {
            log::info!("Start inspecting validator list updates.");
            metrics::validator_list_error_counter().reset();
            let network_postgres = Arc::new(
                PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
            );
            let app_postgres =
                Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
            let redis_client = redis::Client::open(CONFIG.redis.url.as_str()).context(format!(
                "Cannot connect to Redis at URL {}.",
                CONFIG.redis.url
            ))?;
            let mut redis_pubsub_connection =
                redis_client.get_async_connection().await?.into_pubsub();
            let mut redis_data_connection = redis_client.get_async_connection().await?;
            redis_pubsub_connection
                .subscribe(format!(
                    "subvt:{}:validators:publish:finalized_block_number",
                    CONFIG.substrate.chain
                ))
                .await?;
            // keep this to avoid duplicate block processing
            let mut last_finalized_block_number = 0;
            // keep track of validators
            let mut validator_map: HashMap<String, ValidatorDetails> = HashMap::new();
            let last_active_era_index = AtomicU32::new(0);
            let mut pubsub_stream = redis_pubsub_connection.on_message();
            let error: anyhow::Error = loop {
                let maybe_message = pubsub_stream.next().await;
                let payload = if let Some(message) = maybe_message {
                    message.get_payload()
                } else {
                    continue;
                };
                if let Err(error) = payload {
                    break error.into();
                }
                let finalized_block_number: u64 = payload.unwrap();
                if last_finalized_block_number == finalized_block_number {
                    log::warn!(
                        "Skip duplicate finalized block #{}.",
                        finalized_block_number
                    );
                    continue;
                }
                let start = std::time::Instant::now();
                if let Err(error) = self
                    .inspect_validator_list_update(
                        network_postgres.clone(),
                        app_postgres.clone(),
                        &mut redis_data_connection,
                        &mut validator_map,
                        finalized_block_number,
                        &last_active_era_index,
                    )
                    .await
                {
                    metrics::validator_list_error_counter().inc();
                    break error;
                }
                metrics::processed_validator_list_block_number().set(finalized_block_number as i64);
                metrics::validator_list_processing_time_ms()
                    .observe(start.elapsed().as_millis() as f64);
                log::info!(
                    "Completed validator list inspections for block #{}.",
                    finalized_block_number
                );
                last_finalized_block_number = finalized_block_number;
            };
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            log::error!(
                "Error while processing validator list: {:?}. Sleep for {} seconds, then retry.",
                error,
                delay_seconds,
            );
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }
}

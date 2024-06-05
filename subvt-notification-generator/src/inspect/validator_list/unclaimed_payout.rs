use crate::{NotificationGenerator, CONFIG};
use anyhow::Context;
use chrono::Utc;
use redis::aio::MultiplexedConnection as RedisConnection;
use rustc_hash::FxHashMap as HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::{
    app::notification::NotificationTypeCode, substrate::Era, subvt::ValidatorDetails,
};

impl NotificationGenerator {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn inspect_unclaimed_payouts(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        redis_connection: &mut RedisConnection,
        redis_storage_prefix: &str,
        last_active_era_index: &AtomicU32,
        validator_map: &HashMap<String, ValidatorDetails>,
    ) -> anyhow::Result<()> {
        // check era change & unclaimed payouts
        let db_active_era_json: String = redis::cmd("GET")
            .arg(format!("{redis_storage_prefix}:active_era"))
            .query_async(redis_connection)
            .await
            .context("Can't read active era JSON from Redis.")?;
        let active_era: Era = serde_json::from_str(&db_active_era_json)?;
        let era_start = active_era.get_start_date_time();
        let era_elapsed = Utc::now() - era_start;
        if era_elapsed.num_hours()
            >= CONFIG
                .notification_generator
                .unclaimed_payout_check_delay_hours as i64
            && last_active_era_index.load(Ordering::SeqCst) != active_era.index
        {
            if !network_postgres
                .notification_generator_has_processed_era(active_era.index)
                .await?
            {
                log::debug!("Process era #{} for unclaimed payouts.", active_era.index);
                for validator in validator_map.values() {
                    if !validator.unclaimed_era_indices.is_empty() {
                        let rules = app_postgres
                            .get_notification_rules_for_validator(
                                &NotificationTypeCode::ChainValidatorUnclaimedPayout.to_string(),
                                CONFIG.substrate.network_id,
                                &validator.account.id,
                            )
                            .await?;
                        // generate notifications
                        self.generate_notifications(
                            app_postgres.clone(),
                            &rules,
                            &Some(validator.account.id),
                            Some(&validator.unclaimed_era_indices),
                        )
                        .await?;
                    }
                }
                network_postgres
                    .save_notification_generator_processed_era(active_era.index)
                    .await?;
            }
            // and add the era index to processed era indices
            last_active_era_index.store(active_era.index, Ordering::SeqCst);
        }
        Ok(())
    }
}

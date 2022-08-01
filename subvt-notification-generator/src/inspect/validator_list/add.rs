use crate::NotificationGenerator;
use anyhow::Context;
use redis::aio::Connection as RedisConnection;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::str::FromStr;
use std::sync::Arc;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::{crypto::AccountId, subvt::ValidatorDetails};

impl NotificationGenerator {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn add_validators(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        redis_connection: &mut RedisConnection,
        redis_storage_prefix: &str,
        validator_map: &mut HashMap<String, ValidatorDetails>,
        finalized_block_number: u64,
        active_validator_account_ids: &HashSet<String>,
        added_validator_ids: &HashSet<String>,
    ) -> anyhow::Result<()> {
        log::debug!("Checking for new validators.");
        for added_id in added_validator_ids {
            let account_id = AccountId::from_str(added_id)?;
            log::info!("Persist new validator: {}", account_id.to_ss58_check());
            network_postgres
                .save_new_validator_event(&account_id, finalized_block_number)
                .await?;
            // add to validator map
            let validator_prefix = format!(
                "{}:{}:validator:{}",
                redis_storage_prefix,
                if active_validator_account_ids.contains(added_id) {
                    "active"
                } else {
                    "inactive"
                },
                added_id
            );
            let validator = {
                let db_validator_json: String = redis::cmd("GET")
                    .arg(validator_prefix)
                    .query_async(redis_connection)
                    .await
                    .context("Can't read validator JSON from Redis.")?;
                serde_json::from_str::<ValidatorDetails>(&db_validator_json)?
            };
            validator_map.insert(added_id.clone(), validator);
        }
        Ok(())
    }
}

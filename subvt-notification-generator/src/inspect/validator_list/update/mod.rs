use crate::NotificationGenerator;
use anyhow::Context;
use redis::aio::Connection as RedisConnection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::subvt::ValidatorDetails;

mod active;
mod active_next_session;
mod commission;
mod identity;
mod inactive;
mod inactive_next_session;
mod nomination;
mod session_keys;

impl NotificationGenerator {
    /// Runs after each notification from the validator list updater for each validator,
    /// checks for changes in the validator and persists notifications where a rule requires it.
    pub(crate) async fn inspect_validator_changes(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        redis_connection: &mut RedisConnection,
        redis_prefix: &str,
        finalized_block_number: u64,
        last: &ValidatorDetails,
    ) -> anyhow::Result<Option<ValidatorDetails>> {
        let address = &last.account.address;
        // last hash
        let hash = {
            let mut hasher = DefaultHasher::new();
            last.hash(&mut hasher);
            hasher.finish()
        };
        // current hash
        let db_hash: u64 = redis::cmd("GET")
            .arg(format!("{}:hash", redis_prefix))
            .query_async(redis_connection)
            .await
            .context("Can't read validator hash from Redis.")?;
        // return if there's no change in the validator's details
        if hash == db_hash {
            return Ok(None);
        }
        // get current details
        let current = {
            let db_validator_json: String = redis::cmd("GET")
                .arg(redis_prefix)
                .query_async(redis_connection)
                .await
                .context("Can't read validator JSON from Redis.")?;
            serde_json::from_str::<ValidatorDetails>(&db_validator_json)?
        };
        // inspections
        self.inspect_nomination_changes(
            network_postgres.clone(),
            app_postgres.clone(),
            address,
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_active_next_session(
            network_postgres.clone(),
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_inactive_next_session(
            network_postgres.clone(),
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_active(
            network_postgres.clone(),
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_inactive(
            network_postgres.clone(),
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_session_key_change(
            network_postgres.clone(),
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_identity_change(
            network_postgres.clone(),
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_commission_change(
            network_postgres.clone(),
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        self.inspect_onekv_changes(
            network_postgres,
            app_postgres.clone(),
            finalized_block_number,
            last,
            &current,
        )
        .await?;
        Ok(Some(current))
    }
}

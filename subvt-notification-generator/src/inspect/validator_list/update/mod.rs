use crate::NotificationGenerator;
use anyhow::Context;
use redis::Connection as RedisConnection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use subvt_types::subvt::ValidatorDetails;

mod active;
mod active_next_session;
mod inactive;
mod inactive_next_session;
mod nomination;

impl NotificationGenerator {
    /// Runs after each notification from the validator list updater for each validator,
    /// checks for changes in the validator and persists notifications where a rule requires it.
    pub(crate) async fn inspect_validator_changes(
        &self,
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
            .query(redis_connection)
            .context("Can't read validator hash from Redis.")?;
        // return if there's no change in the validator's details
        if hash == db_hash {
            return Ok(None);
        }
        // get current details
        let current = {
            let db_validator_json: String = redis::cmd("GET")
                .arg(redis_prefix)
                .query(redis_connection)
                .context("Can't read validator JSON from Redis.")?;
            serde_json::from_str::<ValidatorDetails>(&db_validator_json)?
        };
        // inspections
        self.inspect_nomination_changes(address, finalized_block_number, last, &current)
            .await?;
        self.inspect_active_next_session(finalized_block_number, last, &current)
            .await?;
        self.inspect_inactive_next_session(finalized_block_number, last, &current)
            .await?;
        self.inspect_active(finalized_block_number, last, &current)
            .await?;
        self.inspect_inactive(finalized_block_number, last, &current)
            .await?;
        self.inspect_onekv_changes(finalized_block_number, last, &current)
            .await?;
        // check 1kv rank, validity and location
        if current.onekv_candidate_record_id.is_some()
            && (current.onekv_candidate_record_id == last.onekv_candidate_record_id)
        {
            // check validity
        }
        Ok(Some(current))
    }
}

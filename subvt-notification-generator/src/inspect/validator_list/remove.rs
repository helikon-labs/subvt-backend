use crate::NotificationGenerator;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::str::FromStr;
use std::sync::Arc;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::{crypto::AccountId, subvt::ValidatorDetails};

impl NotificationGenerator {
    pub(crate) async fn remove_validators(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        validator_map: &mut HashMap<String, ValidatorDetails>,
        finalized_block_number: u64,
        removed_validator_ids: &HashSet<String>,
    ) -> anyhow::Result<()> {
        log::debug!("Checking for removed validators.");
        for removed_id in removed_validator_ids {
            let account_id = AccountId::from_str(removed_id)?;
            log::info!("Remove validator: {}", account_id.to_ss58_check());
            network_postgres
                .save_removed_validator_event(&account_id, finalized_block_number)
                .await?;
            validator_map.remove(removed_id);
        }
        Ok(())
    }
}

use crate::NotificationGenerator;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Nomination;
use subvt_types::subvt::ValidatorDetails;

mod lost_nomination;
mod new_nomination;
mod renomination;

impl NotificationGenerator {
    pub(crate) async fn inspect_nomination_changes(
        &self,
        substrate_client: Arc<SubstrateClient>,
        address: &str,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        let current_nominator_ids: HashSet<AccountId> = current
            .nominations
            .iter()
            .map(|nomination| &nomination.stash_account.id)
            .cloned()
            .collect();
        let last_nominator_ids: HashSet<AccountId> = last
            .nominations
            .iter()
            .map(|nomination| &nomination.stash_account.id)
            .cloned()
            .collect();
        let mut current_nomination_map: HashMap<&AccountId, &Nomination> = HashMap::new();
        for nomination in &current.nominations {
            current_nomination_map.insert(&nomination.stash_account.id, nomination);
        }
        // new nominations
        let new_nominator_ids = &current_nominator_ids - &last_nominator_ids;
        self.inspect_new_nominations(
            substrate_client.clone(),
            address,
            finalized_block_number,
            current,
            &new_nominator_ids,
            &current_nomination_map,
        )
        .await?;
        // lost nominations
        let mut last_nomination_map: HashMap<&AccountId, &Nomination> = HashMap::new();
        for nomination in &last.nominations {
            last_nomination_map.insert(&nomination.stash_account.id, nomination);
        }
        let lost_nominator_ids = &last_nominator_ids - &current_nominator_ids;
        self.inspect_lost_nominations(
            substrate_client.clone(),
            address,
            finalized_block_number,
            current,
            &lost_nominator_ids,
            &last_nomination_map,
        )
        .await?;
        // renominations
        let renominator_ids = &current_nominator_ids - &new_nominator_ids;
        self.inspect_renominations(
            substrate_client,
            address,
            finalized_block_number,
            current,
            &renominator_ids,
            &last_nomination_map,
            &current_nomination_map,
        )
        .await?;
        Ok(())
    }
}

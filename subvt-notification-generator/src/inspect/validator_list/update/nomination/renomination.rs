use crate::{NotificationGenerator, CONFIG};
use std::collections::{HashMap, HashSet};
use subvt_types::{
    app::app_event, app::NotificationTypeCode, crypto::AccountId, substrate::Nomination,
    subvt::ValidatorDetails,
};

impl NotificationGenerator {
    pub(crate) async fn inspect_renominations(
        &self,
        address: &str,
        finalized_block_number: u64,
        current: &ValidatorDetails,
        renominator_ids: &HashSet<AccountId>,
        last_nomination_map: &HashMap<&AccountId, &Nomination>,
        current_nomination_map: &HashMap<&AccountId, &Nomination>,
    ) -> anyhow::Result<()> {
        for renominator_id in renominator_ids {
            let current_nomination = *current_nomination_map.get(&renominator_id).unwrap();
            let prev_nomination = *last_nomination_map.get(&renominator_id).unwrap();
            if current_nomination.stake.active_amount != prev_nomination.stake.active_amount {
                // create app event
                log::debug!(
                    "Changed nomination for {} :: {} :: {} -> {}",
                    address,
                    renominator_id.to_ss58_check(),
                    prev_nomination.stake.active_amount,
                    current_nomination.stake.active_amount,
                );
                let rules = self
                    .app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorNominationAmountChange.to_string(),
                        CONFIG.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                let event = app_event::NominationAmountChange {
                    validator_account_id: current.account.id.clone(),
                    discovered_block_number: finalized_block_number,
                    nominator_stash_account_id: current_nomination.stash_account.id.clone(),
                    prev_active_amount: prev_nomination.stake.active_amount,
                    prev_total_amount: prev_nomination.stake.total_amount,
                    prev_nominee_count: prev_nomination.target_account_ids.len() as u64,
                    active_amount: current_nomination.stake.active_amount,
                    total_amount: current_nomination.stake.total_amount,
                    nominee_count: current_nomination.target_account_ids.len() as u64,
                };
                self.generate_notifications(
                    &rules,
                    finalized_block_number,
                    &current.account.id,
                    Some(&event),
                )
                .await?;
                self.network_postgres
                    .save_nomination_amount_change_event(&event)
                    .await?;
            }
        }
        Ok(())
    }
}

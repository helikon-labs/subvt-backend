use crate::{NotificationGenerator, CONFIG};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::NominationSummary;
use subvt_types::{
    app::app_event, app::notification::NotificationTypeCode, crypto::AccountId,
    subvt::ValidatorDetails,
};

impl NotificationGenerator {
    pub(crate) async fn inspect_renominations(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        address: &str,
        finalized_block_number: u64,
        current: &ValidatorDetails,
        renominator_ids: &HashSet<AccountId>,
        last_nomination_map: &HashMap<&AccountId, &NominationSummary>,
        current_nomination_map: &HashMap<&AccountId, &NominationSummary>,
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
                let rules = app_postgres
                    .get_notification_rules_for_validator(
                        &NotificationTypeCode::ChainValidatorNominationAmountChange.to_string(),
                        CONFIG.substrate.network_id,
                        &current.account.id,
                    )
                    .await?;
                let is_onekv = network_postgres
                    .is_onekv_nominator_account_id(&prev_nomination.stash_account.id)
                    .await?;
                let event = app_event::NominationAmountChange {
                    validator_account_id: current.account.id,
                    discovered_block_number: finalized_block_number,
                    nominator_stash_account_id: current_nomination.stash_account.id,
                    prev_active_amount: prev_nomination.stake.active_amount,
                    prev_total_amount: prev_nomination.stake.total_amount,
                    prev_nominee_count: prev_nomination.nominee_count as u64,
                    active_amount: current_nomination.stake.active_amount,
                    total_amount: current_nomination.stake.total_amount,
                    nominee_count: current_nomination.nominee_count as u64,
                    is_onekv,
                };
                self.generate_notifications(
                    app_postgres.clone(),
                    &rules,
                    &Some(current.account.id),
                    Some(&event),
                )
                .await?;
                /* stop saving this event :: causing extreme storage
                network_postgres
                    .save_nomination_amount_change_event(&event)
                    .await?;
                 */
            }
        }
        Ok(())
    }
}

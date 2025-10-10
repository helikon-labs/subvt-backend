use crate::{NotificationGenerator, CONFIG};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::app_event;
use subvt_types::app::notification::NotificationTypeCode;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::{Balance, NominationSummary};
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_new_nominations(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        address: &str,
        finalized_block_number: u64,
        current: &ValidatorDetails,
        new_nominator_ids: &HashSet<AccountId>,
        current_nomination_map: &HashMap<&AccountId, &NominationSummary>,
    ) -> anyhow::Result<()> {
        for new_nominator_id in new_nominator_ids {
            let new_nomination = *current_nomination_map.get(&new_nominator_id).unwrap();
            log::debug!(
                "New nomination for {} :: {} :: {}",
                address,
                new_nominator_id.to_ss58_check(),
                new_nomination.stake.active_amount,
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorNewNomination.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            let is_onekv = network_postgres
                .is_onekv_nominator_account_id(&new_nomination.stash_account.id)
                .await?;
            let event = app_event::NewNomination {
                validator_account_id: current.account.id,
                discovered_block_number: finalized_block_number,
                nominator_stash_account_id: new_nomination.stash_account.id,
                active_amount: new_nomination.stake.active_amount,
                total_amount: new_nomination.stake.total_amount,
                nominee_count: new_nomination.nominee_count as u64,
                is_onekv,
            };
            for rule in rules {
                if let Some(min_param) = rule.parameters.first() {
                    if let Ok(min_amount) = min_param.value.parse::<Balance>() {
                        if new_nomination.stake.active_amount < min_amount {
                            continue;
                        }
                    }
                }
                self.generate_notifications(
                    app_postgres.clone(),
                    &[rule],
                    &Some(current.account.id),
                    Some(&event),
                )
                .await?;
            }
            network_postgres.save_new_nomination_event(&event).await?;
        }
        Ok(())
    }
}

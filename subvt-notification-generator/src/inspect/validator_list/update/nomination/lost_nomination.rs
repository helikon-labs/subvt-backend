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
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn inspect_lost_nominations(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        address: &str,
        finalized_block_number: u64,
        current: &ValidatorDetails,
        lost_nominator_ids: &HashSet<AccountId>,
        last_nomination_map: &HashMap<&AccountId, &NominationSummary>,
    ) -> anyhow::Result<()> {
        for lost_nominator_id in lost_nominator_ids {
            let lost_nomination = *last_nomination_map.get(&lost_nominator_id).unwrap();
            // create app event
            log::debug!(
                "Lost nomination for {} :: {} :: {}",
                address,
                lost_nominator_id.to_ss58_check(),
                lost_nomination.stake.active_amount,
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorLostNomination.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            let is_onekv = network_postgres
                .is_onekv_nominator_account_id(&lost_nomination.stash_account.id)
                .await?;
            let event = app_event::LostNomination {
                validator_account_id: current.account.id,
                discovered_block_number: finalized_block_number,
                nominator_stash_account_id: lost_nomination.stash_account.id,
                active_amount: lost_nomination.stake.active_amount,
                total_amount: lost_nomination.stake.total_amount,
                nominee_count: lost_nomination.nominee_count as u64,
                is_onekv,
            };
            for rule in rules {
                if let Some(min_param) = rule.parameters.get(0) {
                    if let Ok(min_amount) = min_param.value.parse::<Balance>() {
                        if lost_nomination.stake.active_amount < min_amount {
                            continue;
                        }
                    }
                }
                self.generate_notifications(
                    app_postgres.clone(),
                    &[rule],
                    finalized_block_number,
                    &Some(current.account.id),
                    Some(&event),
                )
                .await?;
            }
            network_postgres.save_lost_nomination_event(&event).await?;
        }
        Ok(())
    }
}

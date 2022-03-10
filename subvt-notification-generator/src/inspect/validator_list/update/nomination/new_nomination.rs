use crate::{NotificationGenerator, CONFIG};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::app_event;
use subvt_types::app::NotificationTypeCode;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::{Balance, Nomination};
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_new_nominations(
        &self,
        substrate_client: Arc<SubstrateClient>,
        address: &str,
        finalized_block_number: u64,
        current: &ValidatorDetails,
        new_nominator_ids: &HashSet<AccountId>,
        current_nomination_map: &HashMap<&AccountId, &Nomination>,
    ) -> anyhow::Result<()> {
        for new_nominator_id in new_nominator_ids {
            let new_nomination = *current_nomination_map.get(&new_nominator_id).unwrap();
            log::debug!(
                "New nomination for {} :: {} :: {}",
                address,
                new_nominator_id.to_ss58_check(),
                new_nomination.stake.active_amount,
            );
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorNewNomination.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            let event = app_event::NewNomination {
                validator_account_id: current.account.id.clone(),
                discovered_block_number: finalized_block_number,
                nominator_stash_account_id: new_nomination.stash_account.id.clone(),
                active_amount: new_nomination.stake.active_amount,
                total_amount: new_nomination.stake.total_amount,
                nominee_count: new_nomination.target_account_ids.len() as u64,
            };
            for rule in rules {
                if let Some(min_param) = rule.parameters.get(0) {
                    if let Ok(min_amount) = min_param.value.parse::<Balance>() {
                        if new_nomination.stake.active_amount < min_amount {
                            continue;
                        }
                    }
                }
                self.generate_notifications(
                    substrate_client.clone(),
                    &[rule],
                    finalized_block_number,
                    &current.account.id,
                    Some(&event),
                )
                .await?;
            }
            self.network_postgres
                .save_new_nomination_event(&event)
                .await?;
        }
        Ok(())
    }
}

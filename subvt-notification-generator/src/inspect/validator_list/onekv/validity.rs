use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::app_event;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_validity_change(
        &self,
        substrate_client: Arc<SubstrateClient>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.onekv_is_valid != last.onekv_is_valid {
            log::debug!(
                "1KV validity of {} changed from {} to {}.",
                current.account.address,
                last.onekv_is_valid.unwrap(),
                current.onekv_is_valid.unwrap(),
            );
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::OneKVValidatorValidityChange.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            let validity_items = self
                .network_postgres
                .get_onekv_candidate_validity_items(current.onekv_candidate_record_id.unwrap())
                .await?;
            self.generate_notifications(
                substrate_client,
                &rules,
                finalized_block_number,
                &current.account.id,
                Some(&app_event::OneKVValidityChange {
                    validator_account_id: current.account.id.clone(),
                    is_valid: current.onekv_is_valid.unwrap(),
                    validity_items,
                }),
            )
            .await?;
            self.network_postgres
                .save_onekv_validity_change_event(
                    &current.account.id,
                    current.onekv_is_valid.unwrap(),
                )
                .await?;
        }
        Ok(())
    }
}

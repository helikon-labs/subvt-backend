use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::app_event;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_validity_change(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.onekv_is_valid.is_none() || last.onekv_is_valid.is_none() {
            return Ok(());
        }
        if current.onekv_is_valid != last.onekv_is_valid {
            log::debug!(
                "1KV validity of {} changed from {} to {}.",
                current.account.address,
                last.onekv_is_valid.unwrap(),
                current.onekv_is_valid.unwrap(),
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::OneKVValidatorValidityChange.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            let validity_items = network_postgres
                .get_onekv_candidate_validity_items(current.onekv_candidate_record_id.unwrap())
                .await?;
            self.generate_notifications(
                app_postgres,
                &rules,
                finalized_block_number,
                &Some(current.account.id),
                Some(&app_event::OneKVValidityChange {
                    validator_account_id: current.account.id,
                    is_valid: current.onekv_is_valid.unwrap(),
                    validity_items,
                }),
            )
            .await?;
            network_postgres
                .save_onekv_validity_change_event(
                    &current.account.id,
                    current.onekv_is_valid.unwrap(),
                )
                .await?;
        }
        Ok(())
    }
}

use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::app_event::CommissionChange;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_commission_change(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        let previous_commission_per_billion = last.preferences.commission_per_billion;
        let current_commission_per_billion = current.preferences.commission_per_billion;
        if current_commission_per_billion != previous_commission_per_billion {
            log::debug!(
                "Commission changed for {} from {} per billion to {}.",
                current.account.address,
                previous_commission_per_billion,
                current_commission_per_billion,
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorCommissionChanged.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            let event = CommissionChange {
                validator_account_id: current.account.id,
                previous_commission_per_billion,
                current_commission_per_billion,
            };
            self.generate_notifications(
                app_postgres,
                &rules,
                finalized_block_number,
                &Some(current.account.id),
                Some(&event),
            )
            .await?;
            network_postgres
                .save_commission_changed(
                    &current.account.id,
                    previous_commission_per_billion,
                    current_commission_per_billion,
                    finalized_block_number,
                )
                .await?;
        }
        Ok(())
    }
}

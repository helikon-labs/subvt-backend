use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::notification::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_active(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.is_active && !last.is_active {
            log::debug!("Now active: {}", current.account.address);
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorActive.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres,
                &rules,
                finalized_block_number,
                &Some(current.account.id),
                if let Some(validator_stake) = &current.validator_stake {
                    Some(validator_stake)
                } else {
                    None
                },
            )
            .await?;
            network_postgres
                .save_active_event(&current.account.id, finalized_block_number)
                .await?;
        }
        Ok(())
    }
}

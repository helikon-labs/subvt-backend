use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{notification::NotificationTypeCode, Block};

impl NotificationGenerator {
    pub(crate) async fn inspect_payout_stakers_extrinsics(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Inspect block #{} for staker payout extrinsics.",
            block.number
        );
        for extrinsic in network_postgres
            .get_payout_stakers_extrinsics_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorPayoutStakers.to_string(),
                    CONFIG.substrate.network_id,
                    &extrinsic.validator_account_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                block.number,
                &Some(extrinsic.validator_account_id),
                Some(&extrinsic.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

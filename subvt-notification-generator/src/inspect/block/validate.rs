use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{notification::NotificationTypeCode, Block};

impl NotificationGenerator {
    /// Checks validation intentions (extrinsics).
    pub(crate) async fn inspect_validate_extrinsics(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for validate extrinsics.", block.number);
        for extrinsic in network_postgres
            .get_validate_extrinsics_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidateExtrinsic.to_string(),
                    CONFIG.substrate.network_id,
                    &extrinsic.stash_account_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                &Some(extrinsic.stash_account_id),
                Some(&extrinsic.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

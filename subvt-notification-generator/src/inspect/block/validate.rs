use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    /// Checks validation intentions (extrinsics).
    pub(crate) async fn process_validate_extrinsics(
        app_postgres: &Arc<PostgreSQLAppStorage>,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        substrate_client: &Arc<SubstrateClient>,
        block: &Block,
    ) -> anyhow::Result<()> {
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
            NotificationGenerator::generate_notifications(
                app_postgres,
                substrate_client,
                &rules,
                block.number,
                &extrinsic.stash_account_id,
                Some(&extrinsic.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

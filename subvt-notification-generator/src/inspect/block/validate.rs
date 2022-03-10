use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    /// Checks validation intentions (extrinsics).
    pub(crate) async fn inspect_validate_extrinsics(
        &self,
        substrate_client: Arc<SubstrateClient>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for validate extrinsics.", block.number);
        for extrinsic in self
            .network_postgres
            .get_validate_extrinsics_in_block(&block.hash)
            .await?
        {
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidateExtrinsic.to_string(),
                    CONFIG.substrate.network_id,
                    &extrinsic.stash_account_id,
                )
                .await?;
            self.generate_notifications(
                substrate_client.clone(),
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

use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    /// Checks validator offline events.
    pub(crate) async fn inspect_offline_offences(
        &self,
        substrate_client: Arc<SubstrateClient>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for offline offences.", block.number);
        for event in self
            .network_postgres
            .get_validator_offline_events_in_block(&block.hash)
            .await?
        {
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorOfflineOffence.to_string(),
                    CONFIG.substrate.network_id,
                    &event.validator_account_id,
                )
                .await?;
            self.generate_notifications(
                substrate_client.clone(),
                &rules,
                block.number,
                &event.validator_account_id,
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

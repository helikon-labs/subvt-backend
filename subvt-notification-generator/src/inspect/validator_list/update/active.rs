use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_active(
        &self,
        substrate_client: Arc<SubstrateClient>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.is_active && !last.is_active {
            log::debug!("Now active: {}", current.account.address);
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorActive.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                substrate_client,
                &rules,
                finalized_block_number,
                &current.account.id,
                if let Some(validator_stake) = &current.validator_stake {
                    Some(validator_stake)
                } else {
                    None
                },
            )
            .await?;
            self.network_postgres
                .save_active_event(&current.account.id, finalized_block_number)
                .await?;
        }
        Ok(())
    }
}

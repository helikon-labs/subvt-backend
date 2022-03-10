use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_active_next_session(
        &self,
        substrate_client: Arc<SubstrateClient>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.active_next_session && !last.active_next_session {
            log::debug!("Active next session: {}", current.account.address,);
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorActiveNextSession.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                substrate_client,
                &rules,
                finalized_block_number,
                &current.account.id,
                None::<&()>,
            )
            .await?;
            self.network_postgres
                .save_active_next_session_event(&current.account.id, finalized_block_number)
                .await?;
        }
        Ok(())
    }
}

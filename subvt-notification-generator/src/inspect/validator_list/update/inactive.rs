use crate::{NotificationGenerator, CONFIG};
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_inactive(
        &self,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if !current.is_active && last.is_active {
            log::debug!("Now inactive: {}", current.account.id.to_ss58_check());
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorInactive.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                &rules,
                finalized_block_number,
                &current.account.id,
                None::<&()>,
            )
            .await?;
            self.network_postgres
                .save_inactive_event(&current.account.id, finalized_block_number)
                .await?;
        }
        Ok(())
    }
}

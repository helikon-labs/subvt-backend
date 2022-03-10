use crate::{NotificationGenerator, CONFIG};
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    /// Checks chilling events.
    pub(crate) async fn inspect_chillings(&self, block: &Block) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for chillings.", block.number);
        for event in self
            .network_postgres
            .get_chilled_events_in_block(&block.hash)
            .await?
        {
            let rules = self
                .app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorChilled.to_string(),
                    CONFIG.substrate.network_id,
                    &event.stash_account_id,
                )
                .await?;
            self.generate_notifications(
                &rules,
                block.number,
                &event.stash_account_id,
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

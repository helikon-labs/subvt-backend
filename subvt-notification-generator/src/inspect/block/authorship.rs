use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    /// Checks if there's any rule watching the author of the block for authorship.
    pub(crate) async fn inspect_block_authorship(
        &self,
        app_postgres: Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Inspect block #{} for authorship notifications.",
            block.number,
        );
        let validator_account_id = if let Some(author_account_id) = &block.author_account_id {
            author_account_id
        } else {
            log::error!("Block ${} author is null.", block.number);
            return Ok(());
        };
        let rules = app_postgres
            .get_notification_rules_for_validator(
                &NotificationTypeCode::ChainValidatorBlockAuthorship.to_string(),
                CONFIG.substrate.network_id,
                validator_account_id,
            )
            .await?;
        self.generate_notifications(
            app_postgres,
            &rules,
            block.number,
            validator_account_id,
            Some(&block.clone()),
        )
        .await?;
        Ok(())
    }
}

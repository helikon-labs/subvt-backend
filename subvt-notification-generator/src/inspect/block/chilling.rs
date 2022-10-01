use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{notification::NotificationTypeCode, Block};

impl NotificationGenerator {
    /// Checks chilling events.
    pub(crate) async fn inspect_chillings(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for chillings.", block.number);
        for event in network_postgres
            .get_chilled_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorChilled.to_string(),
                    CONFIG.substrate.network_id,
                    &event.stash_account_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                block.number,
                &Some(event.stash_account_id),
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    /// Checks chilling events.
    pub(crate) async fn process_chillings(
        app_postgres: &Arc<PostgreSQLAppStorage>,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        substrate_client: &Arc<SubstrateClient>,
        block: &Block,
    ) -> anyhow::Result<()> {
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
            NotificationGenerator::generate_notifications(
                app_postgres,
                substrate_client,
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

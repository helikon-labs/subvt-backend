use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    /// Checks validator offline events.
    pub(crate) async fn process_offline_offences(
        app_postgres: &Arc<PostgreSQLAppStorage>,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        substrate_client: &Arc<SubstrateClient>,
        block: &Block,
    ) -> anyhow::Result<()> {
        for event in network_postgres
            .get_validator_offline_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorOfflineOffence.to_string(),
                    CONFIG.substrate.network_id,
                    &event.validator_account_id,
                )
                .await?;
            NotificationGenerator::generate_notifications(
                app_postgres,
                substrate_client,
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

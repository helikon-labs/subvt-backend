use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{notification::NotificationTypeCode, Block};

impl NotificationGenerator {
    pub(crate) async fn inspect_referenda_events(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        self.inspect_referendum_confirmed_events(&network_postgres, &app_postgres, block)
            .await?;
        Ok(())
    }

    async fn inspect_referendum_confirmed_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for confirmed referenda.", block.number);
        for event in network_postgres
            .get_referendum_confirmed_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumConfirmed.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                block.number,
                &None,
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

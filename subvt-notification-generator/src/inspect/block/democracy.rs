use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{Block, NotificationTypeCode};

impl NotificationGenerator {
    pub(crate) async fn inspect_democracy_events(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        self.inspect_democracy_cancelled_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_democracy_delegated_events(&network_postgres, &app_postgres, block)
            .await?;
        Ok(())
    }

    async fn inspect_democracy_cancelled_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for cancelled referendums.", block.number);
        for event in network_postgres
            .get_democracy_cancelled_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::DemocracyCancelled.to_string(),
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

    async fn inspect_democracy_delegated_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for democracy delegations.", block.number);
        for event in network_postgres
            .get_democracy_delegated_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::DemocracyDelegated.to_string(),
                    CONFIG.substrate.network_id,
                    &event.original_account_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                block.number,
                &Some(event.original_account_id),
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

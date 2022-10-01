use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{notification::NotificationTypeCode, Block};

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
        self.inspect_democracy_not_passed_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_democracy_passed_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_democracy_proposed_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_democracy_seconded_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_democracy_started_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_democracy_undelegated_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_democracy_voted_events(&network_postgres, &app_postgres, block)
            .await?;
        Ok(())
    }

    async fn inspect_democracy_cancelled_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for cancelled referenda.", block.number);
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

    async fn inspect_democracy_not_passed_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for not passed referenda.", block.number);
        for event in network_postgres
            .get_democracy_not_passed_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::DemocracyNotPassed.to_string(),
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

    async fn inspect_democracy_passed_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for passed referenda.", block.number);
        for event in network_postgres
            .get_democracy_passed_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::DemocracyPassed.to_string(),
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

    async fn inspect_democracy_proposed_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Inspect block #{} for new democracy proposals.",
            block.number
        );
        for event in network_postgres
            .get_democracy_proposed_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::DemocracyProposed.to_string(),
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

    async fn inspect_democracy_seconded_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Inspect block #{} for democracy seconded events.",
            block.number
        );
        for event in network_postgres
            .get_democracy_seconded_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::DemocracySeconded.to_string(),
                    CONFIG.substrate.network_id,
                    &event.account_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                block.number,
                &Some(event.account_id),
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }

    async fn inspect_democracy_started_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for started referenda.", block.number);
        for event in network_postgres
            .get_democracy_started_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::DemocracyStarted.to_string(),
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

    async fn inspect_democracy_undelegated_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Inspect block #{} for democracy undelegated events.",
            block.number
        );
        for event in network_postgres
            .get_democracy_undelegated_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::DemocracyUndelegated.to_string(),
                    CONFIG.substrate.network_id,
                    &event.account_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                block.number,
                &Some(event.account_id),
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }

    async fn inspect_democracy_voted_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Inspect block #{} for democracy voted events.",
            block.number
        );
        for event in network_postgres
            .get_democracy_voted_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::DemocracyVoted.to_string(),
                    CONFIG.substrate.network_id,
                    &event.account_id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                block.number,
                &Some(event.account_id),
                Some(&event.clone()),
            )
            .await?;
        }
        Ok(())
    }
}

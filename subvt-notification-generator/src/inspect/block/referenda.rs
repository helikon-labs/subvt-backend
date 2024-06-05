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
        self.inspect_referendum_approved_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_referendum_cancelled_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_referendum_confirmed_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_referendum_decision_started_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_referendum_killed_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_referendum_rejected_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_referendum_submitted_events(&network_postgres, &app_postgres, block)
            .await?;
        self.inspect_referendum_timed_out_events(&network_postgres, &app_postgres, block)
            .await?;
        Ok(())
    }

    async fn inspect_referendum_approved_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for approved referenda.", block.number);
        for event in network_postgres
            .get_referendum_approved_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumApproved.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
        Ok(())
    }

    async fn inspect_referendum_cancelled_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for cancelled referenda.", block.number);
        for event in network_postgres
            .get_referendum_cancelled_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumCancelled.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
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
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
        Ok(())
    }

    async fn inspect_referendum_decision_started_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Inspect block #{} for decision started referenda.",
            block.number
        );
        for event in network_postgres
            .get_referendum_decision_started_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumDecisionStarted.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
        Ok(())
    }

    async fn inspect_referendum_killed_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for killed referenda.", block.number);
        for event in network_postgres
            .get_referendum_killed_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumKilled.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
        Ok(())
    }

    async fn inspect_referendum_rejected_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for rejected referenda.", block.number);
        for event in network_postgres
            .get_referendum_rejected_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumRejected.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
        Ok(())
    }

    async fn inspect_referendum_submitted_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for submitted referenda.", block.number);
        for event in network_postgres
            .get_referendum_submitted_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumSubmitted.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
        Ok(())
    }

    async fn inspect_referendum_timed_out_events(
        &self,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        app_postgres: &Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        log::debug!("Inspect block #{} for timed out referenda.", block.number);
        for event in network_postgres
            .get_referendum_timed_out_events_in_block(&block.hash)
            .await?
        {
            let rules = app_postgres
                .get_notification_rules_by_type(
                    &NotificationTypeCode::ReferendumTimedOut.to_string(),
                    CONFIG.substrate.network_id,
                )
                .await?;
            self.generate_notifications(app_postgres.clone(), &rules, &None, Some(&event.clone()))
                .await?;
        }
        Ok(())
    }
}

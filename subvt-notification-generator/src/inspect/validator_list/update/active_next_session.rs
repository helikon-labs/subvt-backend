use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_active_next_session(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.active_next_session && !last.active_next_session {
            log::debug!("Active next session: {}", current.account.address,);
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorActiveNextSession.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres.clone(),
                &rules,
                finalized_block_number,
                &Some(current.account.id),
                None::<&()>,
            )
            .await?;
            network_postgres
                .save_active_next_session_event(&current.account.id, finalized_block_number)
                .await?;
        }
        Ok(())
    }
}

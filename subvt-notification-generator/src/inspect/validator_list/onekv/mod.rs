use crate::NotificationGenerator;
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::subvt::ValidatorDetails;

mod location;
mod online;
mod rank;
mod validity;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_changes(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.onekv_candidate_record_id.is_none() {
            return Ok(());
        }
        self.inspect_onekv_rank_change(
            network_postgres.clone(),
            app_postgres.clone(),
            last,
            current,
        )
        .await?;
        self.inspect_onekv_location_change(
            network_postgres.clone(),
            app_postgres.clone(),
            last,
            current,
        )
        .await?;
        self.inspect_onekv_validity_change(
            network_postgres.clone(),
            app_postgres.clone(),
            last,
            current,
        )
        .await?;
        self.inspect_onekv_online_status_change(network_postgres, app_postgres, last, current)
            .await?;
        Ok(())
    }
}

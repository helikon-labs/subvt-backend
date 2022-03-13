use crate::NotificationGenerator;
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::subvt::ValidatorDetails;

mod binary_version;
mod location;
mod rank;
mod validity;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_changes(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        substrate_client: Arc<SubstrateClient>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.onekv_candidate_record_id.is_none()
            || (current.onekv_candidate_record_id != last.onekv_candidate_record_id)
        {
            return Ok(());
        }
        self.inspect_onekv_binary_version_change(
            network_postgres.clone(),
            app_postgres.clone(),
            substrate_client.clone(),
            finalized_block_number,
            last,
            current,
        )
        .await?;
        self.inspect_onekv_rank_change(
            network_postgres.clone(),
            app_postgres.clone(),
            substrate_client.clone(),
            finalized_block_number,
            last,
            current,
        )
        .await?;
        self.inspect_onekv_location_change(
            network_postgres.clone(),
            app_postgres.clone(),
            substrate_client.clone(),
            finalized_block_number,
            last,
            current,
        )
        .await?;
        self.inspect_onekv_validity_change(
            network_postgres,
            app_postgres,
            substrate_client,
            finalized_block_number,
            last,
            current,
        )
        .await?;
        Ok(())
    }
}

use crate::NotificationGenerator;
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::Block;

impl NotificationGenerator {
    pub(crate) async fn inspect_democracy_events(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        block: &Block,
    ) -> anyhow::Result<()> {
        self.inspect_democracy_cancelled_events(&network_postgres, &app_postgres, block)
            .await?;
        Ok(())
    }

    async fn inspect_democracy_cancelled_events(
        &self,
        _network_postgres: &Arc<PostgreSQLNetworkStorage>,
        _app_postgres: &Arc<PostgreSQLAppStorage>,
        _block: &Block,
    ) -> anyhow::Result<()> {
        // get cancelled events in block
        // for each :: get notification rules :: just by type
        // generate notifications
        Ok(())
    }
}

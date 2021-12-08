use crate::postgres::PostgreSQLStorage;
use subvt_types::app::Network;

type PostgresNetwork = (
    i32,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl PostgreSQLStorage {
    pub async fn get_networks(&self) -> anyhow::Result<Vec<Network>> {
        let networks: Vec<PostgresNetwork> = sqlx::query_as(
            r#"
            SELECT id, hash, name, app_service_url, live_network_status_service_url, report_service_url, validator_details_service_url, validator_list_service_url
            FROM app_network
            ORDER BY id ASC
            "#
        )
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(networks
            .iter()
            .cloned()
            .map(|db_network| Network {
                id: db_network.0,
                hash: db_network.1,
                name: db_network.2,
                app_service_url: db_network.3,
                live_network_status_service_url: db_network.4,
                report_service_url: db_network.5,
                validator_details_service_url: db_network.6,
                validator_list_service_url: db_network.7,
            })
            .collect())
    }
}

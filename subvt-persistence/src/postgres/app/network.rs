//! Application storage related to the networks supported by SubVT.
use crate::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::db::PostgresNetwork;
use subvt_types::app::Network;

impl PostgreSQLAppStorage {
    pub async fn get_network_by_id(&self, id: u32) -> anyhow::Result<Network> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, hash, name, ss58_prefix, live_network_status_service_url, report_service_url, validator_details_service_url, validator_list_service_url
            FROM app_network
            WHERE id = $1
            "#
        )
            .bind(id as i32)
            .fetch_one(&self.connection_pool)
            .await
            .map(PostgresNetwork::into)?)
    }

    pub async fn network_exists_by_id(&self, id: u32) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_network
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn get_networks(&self) -> anyhow::Result<Vec<Network>> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, hash, name, ss58_prefix, live_network_status_service_url, report_service_url, validator_details_service_url, validator_list_service_url
            FROM app_network
            ORDER BY id ASC
            "#
        )
            .fetch_all(&self.connection_pool)
            .await?
            .iter()
            .cloned()
            .map(PostgresNetwork::into)
            .collect())
    }
}

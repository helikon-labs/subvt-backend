use crate::postgres::PostgreSQLStorage;
use subvt_types::app::{Network, User};

type PostgresNetwork = (
    i32,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl PostgreSQLStorage {
    pub async fn get_networks(&self) -> anyhow::Result<Vec<Network>> {
        let networks: Vec<PostgresNetwork> = sqlx::query_as(
            r#"
            SELECT id, hash, name, live_network_status_service_url, report_service_url, validator_details_service_url, validator_list_service_url
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
                live_network_status_service_url: db_network.3,
                report_service_url: db_network.4,
                validator_details_service_url: db_network.5,
                validator_list_service_url: db_network.6,
            })
            .collect())
    }

    pub async fn save_user(&self, user: &User) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO app_user (public_key_hex)
            VALUES ($1)
            RETURNING id
            "#,
        )
        .bind(&user.public_key_hex)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0)
    }

    pub async fn user_exists(&self, public_key_hex: &str) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user
            WHERE public_key_hex = $1
            "#,
        )
        .bind(public_key_hex)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }
}

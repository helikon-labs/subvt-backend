use log::debug;
use sqlx::{Pool, Postgres};
use subvt_config::Config;

pub mod network;
pub mod notification;
pub mod notification_channel;
pub mod notification_type;
pub mod user;

pub struct PostgreSQLAppStorage {
    _uri: String,
    connection_pool: Pool<Postgres>,
}

impl PostgreSQLAppStorage {
    pub async fn new(config: &Config, uri: String) -> anyhow::Result<PostgreSQLAppStorage> {
        debug!("Establishing application database connection pool...");
        let connection_pool = sqlx::postgres::PgPoolOptions::new()
            .connect_timeout(std::time::Duration::from_secs(
                config.network_postgres.connection_timeout_seconds,
            ))
            .max_connections(config.network_postgres.pool_max_connections)
            .connect(&uri)
            .await?;
        debug!("Application database connection pool established.");
        Ok(PostgreSQLAppStorage {
            _uri: uri,
            connection_pool,
        })
    }
}

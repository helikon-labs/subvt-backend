//! PostgreSQL persistence for SubVT application-related storage.
//! The application database is separate from the databases for each supported network.
use sqlx::{Pool, Postgres};
use std::time::Duration;
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
        log::info!("Establishing application database connection pool.");
        let connection_pool = sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(
                config.network_postgres.connection_timeout_seconds,
            ))
            .max_connections(config.network_postgres.pool_max_connections)
            .connect(&uri)
            .await?;
        log::info!("Application database connection pool established.");
        Ok(PostgreSQLAppStorage {
            _uri: uri,
            connection_pool,
        })
    }
}

//! Storage related to a network supported by SubVT.
//! Each supported network has a separate database.
use sqlx::{Pool, Postgres};
use std::time::Duration;
use subvt_config::Config;

pub mod account;
pub mod app_event;
pub mod block;
pub mod epoch;
pub mod era;
pub mod error_log;
pub mod event;
pub mod extrinsic;
pub mod kline;
pub mod nft;
pub mod notify;
pub mod onekv;
pub mod para;
pub mod referendum;
pub mod report;
pub mod staking;
pub mod telegram;
pub mod telemetry;

pub struct PostgreSQLNetworkStorage {
    uri: String,
    connection_pool: Pool<Postgres>,
}

impl PostgreSQLNetworkStorage {
    pub async fn new(config: &Config, uri: String) -> anyhow::Result<PostgreSQLNetworkStorage> {
        log::info!("Establishing network database connection pool...");
        let connection_pool = sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(
                config.network_postgres.connection_timeout_seconds,
            ))
            .max_connections(config.network_postgres.pool_max_connections)
            .connect(&uri)
            .await?;
        log::info!("Network database connection pool established.");
        Ok(PostgreSQLNetworkStorage {
            uri,
            connection_pool,
        })
    }
}

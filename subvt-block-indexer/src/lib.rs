//! Indexes historical block data to the PostreSQL database instance.

use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug};
use sqlx::{Pool, Postgres};
use subvt_config::Config;
use subvt_service_common::Service;
use chrono::Utc;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct BlockIndexer;

impl BlockIndexer {
    async fn establish_db_connection() -> anyhow::Result<Pool<Postgres>> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&CONFIG.get_postgres_url()).await?;
        Ok(pool)
    }
}

#[async_trait]
impl Service for BlockIndexer {
    async fn run(&'static self) -> anyhow::Result<()> {
        debug!("Will get database connection.");
        let pool = BlockIndexer::establish_db_connection().await?;
        debug!("Got database connection.");
        sqlx::query(
            "INSERT INTO account (id, discovered_at) VALUES ($1, $2)"
        ).bind("ASD").bind(Utc::now()).fetch_one(&pool).await?;
        Ok(())
    }
}
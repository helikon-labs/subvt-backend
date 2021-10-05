//! Indexes historical block data into the PostreSQL database instance.

use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, error};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::BlockHeader;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct BlockIndexer;

impl BlockIndexer {
    async fn establish_db_connection() -> anyhow::Result<Pool<Postgres>> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&CONFIG.get_postgres_url())
            .await?;
        Ok(pool)
    }

    async fn index_block(
        &self,
        substrate_client: &SubstrateClient,
        _db_connection_pool: &Pool<Postgres>,
        new_block_header: &BlockHeader,
    ) -> anyhow::Result<()> {
        let block_number = new_block_header.get_number()?;
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        // get block events
        let events = substrate_client.get_block_events(&block_hash).await?;
        debug!("Got #{} events for block #{}.", events.len(), block_number);
        // check runtime version - reload meta if changed
        // get extrinsics & set timestamp
        // check era - if changed
        //  get active and inactive validator list
        //  total past era points, past era points per validator
        //

        // rewards, offences, other events

        // write to database -
        Ok(())
    }
}

#[async_trait]
impl Service for BlockIndexer {
    async fn run(&'static self) -> anyhow::Result<()> {
        /*
        let mut parent_account = Account::default();
        let parent_account_id = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2];
        parent_account.id = parent_account_id.into();
        parent_account.discovered_at = Some(Utc::now().timestamp_millis() / 1000);
        sqlx::query(
            "INSERT INTO account (id, discovered_at) VALUES ($1, $2)"
        )
            .bind(parent_account.id.to_string())
            .bind(parent_account.discovered_at)
            .fetch_one(&pool)
            .await?;

        let mut parent_account_identity = IdentityRegistration::default();
        parent_account_identity.display = Some("parent account display".to_string());
        parent_account_identity.twitter = Some("parent_account".to_string());
        parent_account_identity.confirmed = true;

        let mut account = Account::default();
        account.id = AccountId::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1]);
        sqlx::query(
            "INSERT INTO account (id, parent_id) VALUES ($1, $2)"
        )
            .bind(account.id.to_string())
            .bind(parent_account.id.to_string())
            .fetch_one(&pool).await?;
         */

        loop {
            let substrate_client = Arc::new(SubstrateClient::new(&CONFIG).await?);
            substrate_client.metadata.log_all_calls();
            substrate_client.metadata.log_all_events();
            let db_connection_pool = Arc::new(BlockIndexer::establish_db_connection().await?);

            debug!("Database connection pool established.");
            substrate_client.subscribe_to_new_blocks(|new_block_header| {
                let substrate_client = Arc::clone(&substrate_client);
                let db_connection_pool = Arc::clone(&db_connection_pool);
                tokio::spawn(async move {
                    let update_result = self.index_block(
                        &substrate_client,
                        &db_connection_pool,
                        &new_block_header,
                    ).await;
                    match update_result {
                        Ok(_) => (),
                        Err(error) => {
                            error!(
                                "{:?}",
                                error,
                            );
                            error!(
                                "Live network status update failed for block #{}. Will try again with the next block.",
                                new_block_header.get_number().unwrap_or(0),
                            );
                        }
                    }
                });
            }).await?;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            error!(
                "New block subscription exited. Will refresh connection and subscription after {} seconds.",
                delay_seconds
            );
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }
}

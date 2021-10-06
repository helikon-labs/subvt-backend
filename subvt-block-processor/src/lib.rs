//! Indexes historical block data into the PostreSQL database instance.

use async_lock::Mutex;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, error};
use sqlx::{Pool, Postgres};
use std::sync::{Arc, RwLock};
use subvt_config::Config;
use subvt_service_common::Service;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::BlockHeader;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct BlockProcessor;

#[derive(Default)]
struct RuntimeInformation {
    pub era_index: u32,
    pub epoch_index: u64,
    pub runtime_version: u32,
    pub processed_block_number: u64,
}

impl BlockProcessor {
    async fn establish_db_connection() -> anyhow::Result<Pool<Postgres>> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&CONFIG.get_postgres_url())
            .await?;
        Ok(pool)
    }

    async fn persist_era_validators(
        substrate_client: &SubstrateClient,
        db_connection_pool: &Pool<Postgres>,
        era_index: u32,
        block_hash: &str,
    ) -> anyhow::Result<()> {
        debug!("Persist era #{} validators.", era_index);
        // all validator ids
        let all_validator_account_ids = substrate_client
            .get_all_validator_account_ids(block_hash)
            .await?;
        // active validator ids
        let active_validator_account_ids = substrate_client
            .get_active_validator_account_ids(block_hash)
            .await?;
        let mut db_tx = db_connection_pool.begin().await?;
        for validator_account_id in all_validator_account_ids {
            // create validator account (if not exists)
            sqlx::query(
                r#"
                INSERT INTO account (id)
                VALUES ($1)
                ON CONFLICT (id) DO NOTHING
                RETURNING id
                "#,
            )
            .bind(validator_account_id.to_string())
            .execute(&mut db_tx)
            .await?;
            let is_active = active_validator_account_ids.contains(&validator_account_id);
            // create record (if not exists)
            sqlx::query(
                r#"
                INSERT INTO era_validator (era_index, validator_account_id, is_active)
                VALUES ($1, $2, $3)
                ON CONFLICT (era_index, validator_account_id) DO NOTHING
                RETURNING id
                "#,
            )
            .bind(era_index)
            .bind(validator_account_id.to_string())
            .bind(is_active)
            .execute(&mut db_tx)
            .await?;
        }
        db_tx.commit().await?;
        debug!("Persisted era #{} validators.", era_index);
        Ok(())
    }

    async fn persist_era_reward_points(
        substrate_client: &SubstrateClient,
        db_connection_pool: &Pool<Postgres>,
        era_index: u32,
    ) -> anyhow::Result<()> {
        // check if era exists
        let era_record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(index) FROM era
            WHERE index = $1
            "#,
        )
        .bind(era_index)
        .fetch_one(db_connection_pool)
        .await?;
        if era_record_count.0 == 0 {
            debug!("ERA {} DOES NOT EXIST.", era_index);
            return Ok(());
        }
        debug!("ERA {} EXISTS.", era_index);
        let era_reward_points = substrate_client.get_era_reward_points(era_index).await?;
        // update era record
        sqlx::query(
            r#"
            UPDATE era SET reward_points_total = $1
            WHERE index = $2
            "#,
        )
        .bind(era_reward_points.total)
        .bind(era_index)
        .execute(db_connection_pool)
        .await?;
        for (validator_account_id, reward) in era_reward_points.individual {
            let account_id_bytes: &[u8; 32] = validator_account_id.as_ref();
            let account_id = AccountId::new(*account_id_bytes);
            sqlx::query(
                r#"
                UPDATE era_validator SET reward_points = $1, last_updated = now()
                WHERE era_index = $2 AND validator_account_id = $3
                "#,
            )
            .bind(reward)
            .bind(era_index)
            .bind(account_id.to_string())
            .execute(db_connection_pool)
            .await?;
        }
        Ok(())
    }

    async fn process_block(
        &self,
        substrate_client: &SubstrateClient,
        runtime_information: &Arc<RwLock<RuntimeInformation>>,
        db_connection_pool: &Pool<Postgres>,
        block_header: &BlockHeader,
    ) -> anyhow::Result<()> {
        let block_number = block_header.get_number()?;
        debug!("Process block #{}.", block_number);
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let runtime_upgrade_info = substrate_client
            .get_last_runtime_upgrade_info(&block_hash)
            .await?;
        // check metadata version
        let last_runtime_version = { runtime_information.read().unwrap().runtime_version };
        if last_runtime_version != 0 && last_runtime_version != runtime_upgrade_info.spec_version {
            // update metadata & make checks
        }
        {
            runtime_information.write().unwrap().runtime_version =
                runtime_upgrade_info.spec_version;
        }
        let (last_era_index, last_epoch_index) = {
            let runtime_information = runtime_information.read().unwrap();
            (
                runtime_information.era_index,
                runtime_information.epoch_index,
            )
        };
        let active_era = substrate_client.get_active_era(&block_hash).await?;
        let current_epoch = substrate_client.get_current_epoch(&block_hash).await?;
        if last_epoch_index != current_epoch.index {
            debug!("New epoch. Persist era and epoch if not exists.");
            sqlx::query(
                r#"
                INSERT INTO era (index, start_timestamp, end_timestamp)
                VALUES ($1, $2, $3)
                ON CONFLICT (index) DO NOTHING
                RETURNING index
                "#,
            )
            .bind(active_era.index)
            .bind(active_era.start_timestamp as u32)
            .bind(active_era.end_timestamp as u32)
            .fetch_optional(db_connection_pool)
            .await?;
            // check if current epoch is persisted - persist if not
            sqlx::query(
                r#"
                INSERT INTO epoch (index, era_index, start_block_number, start_timestamp, end_timestamp)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (index) DO NOTHING
                RETURNING index
                "#)
                .bind(current_epoch.index as u32)
                .bind(active_era.index)
                .bind(current_epoch.start_block_number)
                .bind(current_epoch.start_timestamp as u32)
                .bind(current_epoch.end_timestamp as u32)
                .fetch_optional(db_connection_pool)
                .await?;
        }
        if last_era_index != active_era.index {
            // check if last era's validators are persisted
            BlockProcessor::persist_era_validators(
                substrate_client,
                db_connection_pool,
                active_era.index,
                block_hash.as_str(),
            )
            .await?;
            // persist last era reward points
            BlockProcessor::persist_era_reward_points(
                substrate_client,
                db_connection_pool,
                active_era.index,
            )
            .await?;
        }
        {
            let mut runtime_information = runtime_information.write().unwrap();
            runtime_information.era_index = active_era.index;
            runtime_information.epoch_index = current_epoch.index;
        }

        let events = substrate_client.get_block_events(&block_hash).await?;
        debug!("Got #{} events for block #{}.", events.len(), block_number);
        let extrinsics = substrate_client.get_block_extrinsics(&block_hash).await?;
        debug!(
            "Got #{} extrinsics for block #{}.",
            extrinsics.len(),
            block_number
        );
        // get author account + persist self and parent if exists
        // get era + persist
        // get epoch/session + persist
        // persist block
        {
            runtime_information.write().unwrap().processed_block_number = block_number;
        }
        Ok(())
    }
}

#[async_trait]
impl Service for BlockProcessor {
    async fn run(&'static self) -> anyhow::Result<()> {
        loop {
            let substrate_client = Arc::new(SubstrateClient::new(&CONFIG).await?);
            let runtime_information = Arc::new(RwLock::new(RuntimeInformation::default()));
            let busy_lock = Arc::new(Mutex::new(()));
            substrate_client.metadata.log_all_calls();
            substrate_client.metadata.log_all_events();
            debug!("Getting database connection...");
            let db_connection_pool = Arc::new(BlockProcessor::establish_db_connection().await?);
            debug!("Database connection pool established.");
            substrate_client.subscribe_to_finalized_blocks(|finalized_block_header| {
                let substrate_client = substrate_client.clone();
                let runtime_information = runtime_information.clone();
                let db_connection_pool = db_connection_pool.clone();
                let busy_lock = busy_lock.clone();
                tokio::spawn(async move {
                    let _lock = busy_lock.lock().await;
                    let update_result = self.process_block(
                        &substrate_client,
                        &runtime_information,
                        &db_connection_pool,
                        &finalized_block_header,
                    ).await;
                    match update_result {
                        Ok(_) => (),
                        Err(error) => {
                            error!("{:?}", error);
                            error!(
                                "Block processing failed for finalized block #{}. Will try again with the next block.",
                                finalized_block_header.get_number().unwrap_or(0),
                            );
                        }
                    }
                });
            }).await?;
            let delay_seconds = CONFIG.common.recovery_retry_seconds;
            error!(
                "Finalized block subscription exited. Will refresh connection and subscription after {} seconds.",
                delay_seconds
            );
            std::thread::sleep(std::time::Duration::from_secs(delay_seconds));
        }
    }
}

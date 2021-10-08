use log::debug;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::{BlockHeader, Epoch, Era};

pub struct PostgreSQLStorage {
    connection_pool: Pool<Postgres>,
}

impl PostgreSQLStorage {
    pub async fn new(config: &Config) -> anyhow::Result<PostgreSQLStorage> {
        debug!("Establishing database connection pool...");
        let connection_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.postgres.pool_max_connections)
            .connect(&config.get_postgres_url())
            .await?;
        debug!("Database connection pool established.");
        Ok(PostgreSQLStorage { connection_pool })
    }
}

impl PostgreSQLStorage {
    pub async fn save_account(&self, account_id: &AccountId) -> anyhow::Result<Option<AccountId>> {
        let maybe_result: Option<(String,)> = sqlx::query_as(
            r#"
            INSERT INTO account (id)
            VALUES ($1)
            ON CONFLICT (id) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(AccountId::from_str(&result.0)?))
        } else {
            Ok(None)
        }
    }

    pub async fn save_era(&self, era: &Era) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            INSERT INTO era (index, start_timestamp, end_timestamp)
            VALUES ($1, $2, $3)
            ON CONFLICT (index) DO NOTHING
            RETURNING index
            "#,
        )
        .bind(era.index)
        .bind(era.start_timestamp as u32)
        .bind(era.end_timestamp as u32)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_epoch(&self, era_index: u32, epoch: &Epoch) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64, )> = sqlx::query_as(
            r#"
                INSERT INTO epoch (index, era_index, start_block_number, start_timestamp, end_timestamp)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (index) DO NOTHING
                RETURNING index
                "#)
            .bind(epoch.index as u32)
            .bind(era_index)
            .bind(epoch.start_block_number)
            .bind(epoch.start_timestamp as u32)
            .bind(epoch.end_timestamp as u32)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_era_validator(
        &self,
        era_index: u32,
        validator_account_id: &AccountId,
        is_active: bool,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
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
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn era_exists(&self, era_index: u32) -> anyhow::Result<bool> {
        let era_record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(index) FROM era
            WHERE index = $1
            "#,
        )
        .bind(era_index)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(era_record_count.0 > 0)
    }

    pub async fn update_era_reward_points(
        &self,
        era_index: u32,
        reward_points_total: u32,
    ) -> anyhow::Result<Option<i64>> {
        let maybe_result: Option<(i64,)> = sqlx::query_as(
            r#"
            UPDATE era SET reward_points_total = $1, last_updated = now()
            WHERE index = $2
            RETURNING index
            "#,
        )
        .bind(reward_points_total)
        .bind(era_index)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn update_era_validator_reward_points(
        &self,
        era_index: u32,
        validator_account_id: &AccountId,
        reward_points: u32,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE era_validator SET reward_points = $1, last_updated = now()
            WHERE era_index = $2 AND validator_account_id = $3
            RETURNING id
            "#,
        )
        .bind(reward_points)
        .bind(era_index)
        .bind(validator_account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_finalized_block(
        &self,
        block_hash: &str,
        block_header: &BlockHeader,
        block_timestamp: Option<u32>,
        author_account_id_hex: Option<String>,
        (era_index, epoch_index): (u32, u32),
        (metadata_version, runtime_version): (i16, i16),
    ) -> anyhow::Result<Option<String>> {
        let maybe_result: Option<(String, )> = sqlx::query_as(
            r#"
            INSERT INTO block (hash, number, timestamp, author_account_id, era_index, epoch_index, parent_hash, state_root, extrinsics_root, is_finalized, metadata_version, runtime_version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (hash) DO NOTHING
            RETURNING hash
            "#)
            .bind(block_hash)
            .bind(block_header.get_number()? as u32)
            .bind(block_timestamp)
            .bind(author_account_id_hex)
            .bind(era_index)
            .bind(epoch_index)
            .bind(&block_header.parent_hash)
            .bind(&block_header.state_root)
            .bind(&block_header.extrinsics_root)
            .bind(true)
            .bind(metadata_version)
            .bind(runtime_version)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_heartbeart_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        validator_account_id: &AccountId,
        era_index: u32,
        epoch_index: u32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO event_im_online_heartbeat_received (block_hash, extrinsic_index, account_id, era_index, epoch_index)
            VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(validator_account_id.to_string())
            .bind(era_index)
            .bind(epoch_index)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

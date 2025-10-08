//! Storage related to a network supported by SubVT.
//! Each supported network has a separate database.
use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::db::PostgresBlock;
use subvt_types::app::Block;
use subvt_types::{crypto::AccountId, substrate::BlockHeader};

impl PostgreSQLNetworkStorage {
    #[allow(clippy::too_many_arguments)]
    pub async fn save_finalized_block(
        &self,
        chain_type: &str,
        block_hash: &str,
        block_header: &BlockHeader,
        block_timestamp: u64,
        maybe_author_account_id: Option<AccountId>,
        (era_index, epoch_index): (u32, u32),
        (metadata_version, runtime_version): (i16, i16),
    ) -> anyhow::Result<Option<String>> {
        let mut maybe_author_account_id_hex: Option<String> = None;
        if let Some(author_account_id) = maybe_author_account_id {
            maybe_author_account_id_hex = Some(author_account_id.to_string());
            self.save_account(&author_account_id).await?;
        }
        let maybe_result: Option<(String, )> = sqlx::query_as(
            r#"
            INSERT INTO sub_block (hash, chain_type, number, timestamp, author_account_id, era_index, epoch_index, parent_hash, state_root, extrinsics_root, is_finalized, metadata_version, runtime_version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (hash) DO NOTHING
            RETURNING hash
            "#)
            .bind(block_hash)
            .bind(chain_type)
            .bind(block_header.get_number()? as i64)
            .bind(block_timestamp as i64)
            .bind(maybe_author_account_id_hex)
            .bind(era_index as i64)
            .bind(epoch_index as i64)
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

    pub async fn get_block_hash(&self, block_number: u64) -> anyhow::Result<Option<String>> {
        Ok(sqlx::query_as(
            r#"
            SELECT hash FROM sub_block
            WHERE "number" = $1
            "#,
        )
        .bind(block_number as i64)
        .fetch_optional(&self.connection_pool)
        .await?
        .map(|hash: (String,)| hash.0))
    }

    pub async fn get_block_by_number(&self, chain_type: &str, block_number: u64) -> anyhow::Result<Option<Block>> {
        let maybe_db_block: Option<PostgresBlock> = sqlx::query_as(
            r#"
            SELECT hash, number, timestamp, author_account_id, era_index, epoch_index, is_finalized, metadata_version, runtime_version
            FROM sub_block
            WHERE "number" = $1 AND chain_type = $2
            "#,
        )
            .bind(block_number as i64)
            .bind(chain_type)
            .fetch_optional(&self.connection_pool)
            .await?;
        match maybe_db_block {
            Some(db_block) => Ok(Some(Block::from(db_block)?)),
            None => Ok(None),
        }
    }

    pub async fn get_processed_block_height(&self, chain_type: &str) -> anyhow::Result<u64> {
        let processed_block_height: (i64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(MAX(number), 0) from sub_block WHERE chain_type = $1
            "#,
        )
        .bind(chain_type)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(processed_block_height.0 as u64)
    }

    pub async fn get_number_of_blocks_in_epoch_by_validator(
        &self,
        epoch_index: u64,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<u32> {
        let number_of_blocks: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT hash) from sub_block
            WHERE epoch_index = $1
            AND author_account_id = $2
            "#,
        )
        .bind(epoch_index as i64)
        .bind(validator_account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(number_of_blocks.0 as u32)
    }
}

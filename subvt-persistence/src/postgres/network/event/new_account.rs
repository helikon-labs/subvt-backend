use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_new_account_event(
        &self,
        block_hash: &str,
        block_number: u64,
        block_timestamp: u64,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<String>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_new_account (block_hash, extrinsic_index, event_index, account_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(block_hash, event_index) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index)
        .bind(event_index)
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        if maybe_result.is_none() {
            return Ok(None);
        }
        // update account
        let maybe_result: Option<(String,)> = sqlx::query_as(
            r#"
            UPDATE sub_account SET discovered_at_block_hash = $1, discovered_at_block_number = $2, discovered_at = $3, updated_at = now()
            WHERE id = $4
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp as i64)
            .bind(account_id.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn update_new_account_event_batch_index(
        &self,
        block_hash: &str,
        batch_index: &Option<String>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_new_account
            SET batch_index = $1
            WHERE block_hash = $2 AND event_index = $3
            "#,
        )
        .bind(batch_index)
        .bind(block_hash)
        .bind(event_index)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}

use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_killed_account_event(
        &self,
        block_hash: &str,
        block_number: u64,
        block_timestamp: Option<u64>,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<String>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_killed_account (block_hash, extrinsic_index, event_index, account_id)
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
            UPDATE sub_account SET killed_at_block_hash = $1, discovered_at_block_number = $2, discovered_at = $3, updated_at = now()
            WHERE id = $4
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(block_timestamp.map(|timestamp| timestamp as i64))
            .bind(account_id.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }
}

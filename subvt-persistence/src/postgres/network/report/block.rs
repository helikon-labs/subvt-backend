use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;
use subvt_types::report::BlockSummary;

impl PostgreSQLNetworkStorage {
    pub async fn get_blocks_by_validator_in_session(
        &self,
        session_index: u64,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<Vec<BlockSummary>> {
        let blocks: Vec<(i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT number, hash, timestamp
            FROM sub_block
            WHERE epoch_index = $1
            AND author_account_id = $2
            ORDER BY number ASC
            "#,
        )
        .bind(session_index as i64)
        .bind(validator_account_id.to_string())
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(blocks
            .iter()
            .map(|block| BlockSummary {
                number: block.0 as u64,
                hash: block.1.clone(),
                timestamp: block.2 as u64,
            })
            .collect())
    }
}

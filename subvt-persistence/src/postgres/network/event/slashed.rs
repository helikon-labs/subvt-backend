use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::{crypto::AccountId, substrate::Balance};

impl PostgreSQLNetworkStorage {
    pub async fn save_slashed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        validator_account_id: &AccountId,
        amount: Balance,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(validator_account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_slashed (block_hash, extrinsic_index, event_index, validator_account_id, amount)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(block_hash, event_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(validator_account_id.to_string())
            .bind(amount.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }
}

use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_commission_changed(
        &self,
        validator_account_id: &AccountId,
        previous_commission_per_billion: u32,
        current_commission_per_billion: u32,
        discovered_block_number: u64,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_validator_commission_changed (validator_account_id, previous_commission_per_billion, current_commission_per_billion, discovered_block_number)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
            .bind(validator_account_id.to_string())
            .bind(previous_commission_per_billion as i64)
            .bind(current_commission_per_billion as i64)
            .bind(discovered_block_number as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }
}

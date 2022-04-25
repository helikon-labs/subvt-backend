use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_session_keys_changed(
        &self,
        validator_account_id: &AccountId,
        session_keys: &str,
        discovered_block_number: u64,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_validator_session_keys_changed (validator_account_id, session_keys, discovered_block_number)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
            .bind(validator_account_id.to_string())
            .bind(session_keys)
            .bind(discovered_block_number as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }
}

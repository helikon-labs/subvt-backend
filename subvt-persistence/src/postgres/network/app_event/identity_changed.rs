use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_identity_changed(
        &self,
        validator_account_id: &AccountId,
        identity_display: &Option<String>,
        discovered_block_number: u64,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_validator_identity_changed (validator_account_id, identity_display, discovered_block_number)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
            .bind(validator_account_id.to_string())
            .bind(identity_display)
            .bind(discovered_block_number as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }
}

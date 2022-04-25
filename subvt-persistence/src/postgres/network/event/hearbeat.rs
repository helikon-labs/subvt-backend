use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_validator_heartbeart_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        session_index: i64,
        im_online_key_hex_string: &str,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        self.save_account(validator_account_id).await?;
        sqlx::query(
            r#"
            INSERT INTO sub_event_heartbeat_received (block_hash, extrinsic_index, event_index, session_index, im_online_key, validator_account_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (block_hash, event_index, validator_account_id) DO NOTHING
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(session_index)
            .bind(im_online_key_hex_string)
            .bind(validator_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_payout_started_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        era_index: u32,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_event_payout_started (block_hash, extrinsic_index, event_index, era_index, validator_account_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (block_hash, event_index) DO NOTHING
            "#)
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(era_index)
            .bind(validator_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

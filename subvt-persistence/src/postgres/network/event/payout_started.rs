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

    pub async fn update_payout_started_event_batch_index(
        &self,
        block_hash: &str,
        batch_index: &Option<String>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_payout_started
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

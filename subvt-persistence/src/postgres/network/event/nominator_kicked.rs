use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_nominator_kicked_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        validator_account_id: &AccountId,
        nominator_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        self.save_account(validator_account_id).await?;
        self.save_account(nominator_account_id).await?;
        sqlx::query(
            r#"
            INSERT INTO sub_event_nominator_kicked (block_hash, extrinsic_index, event_index, validator_account_id, nominator_account_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(block_hash, event_index) DO NOTHING
            "#)
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(validator_account_id.to_string())
            .bind(nominator_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

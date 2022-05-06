use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::Balance;

impl PostgreSQLNetworkStorage {
    pub async fn save_era_paid_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        era_index: u32,
        validator_payout: Balance,
        remainder: Balance,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_event_era_paid (block_hash, extrinsic_index, event_index, era_index, validator_payout, remainder)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (block_hash, event_index, era_index) DO NOTHING
            "#)
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(era_index)
            .bind(validator_payout.to_string())
            .bind(remainder.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn update_era_paid_event_nesting_index(
        &self,
        block_hash: &str,
        maybe_nesting_index: &Option<String>,
        event_index: i32,
        era_index: u32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_era_paid
            SET nesting_index = $1
            WHERE block_hash = $2 AND event_index = $3 AND era_index = $4
            "#,
        )
        .bind(maybe_nesting_index)
        .bind(block_hash)
        .bind(event_index)
        .bind(era_index)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}

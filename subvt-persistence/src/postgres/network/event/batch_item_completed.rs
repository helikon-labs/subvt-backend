use crate::postgres::network::PostgreSQLNetworkStorage;

impl PostgreSQLNetworkStorage {
    pub async fn save_batch_item_completed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_event_batch_item_completed (block_hash, extrinsic_index, event_index)
            VALUES ($1, $2, $3)
            ON CONFLICT (block_hash, event_index) DO NOTHING
            "#,
        )
        .bind(block_hash)
        .bind(extrinsic_index)
        .bind(event_index)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn update_batch_item_completed_event_nesting_index(
        &self,
        block_hash: &str,
        maybe_nesting_index: &Option<String>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_batch_item_completed
            SET nesting_index = $1
            WHERE block_hash = $2 AND event_index = $3
            "#,
        )
        .bind(maybe_nesting_index)
        .bind(block_hash)
        .bind(event_index)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}

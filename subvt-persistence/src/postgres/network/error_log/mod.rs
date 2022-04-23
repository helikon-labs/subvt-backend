use crate::postgres::network::PostgreSQLNetworkStorage;

impl PostgreSQLNetworkStorage {
    pub async fn save_extrinsic_process_error_log(
        &self,
        block_hash: &str,
        block_number: u64,
        extrinsic_index: usize,
        ty: &str,
        error_log: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_error_log_process_extrinsic (block_hash, block_number, extrinsic_index, type, error_log)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
            .bind(block_hash)
            .bind(block_number as i64)
            .bind(extrinsic_index as i32)
            .bind(ty)
            .bind(error_log)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn get_extrinsic_process_error_log_count(&self) -> anyhow::Result<u64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id)
            FROM sub_error_log_process_extrinsic
            "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count.0 as u64)
    }

    pub async fn save_event_process_error_log(
        &self,
        block_hash: &str,
        block_number: u64,
        event_index: usize,
        ty: &str,
        error_log: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_error_log_process_event (block_hash, block_number, event_index, type, error_log)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(block_hash)
        .bind(block_number as i64)
        .bind(event_index as i32)
        .bind(ty)
        .bind(error_log)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn get_event_process_error_log_count(&self) -> anyhow::Result<u64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id)
            FROM sub_error_log_process_event
            "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count.0 as u64)
    }
}

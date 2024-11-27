use crate::postgres::network::PostgreSQLNetworkStorage;

impl PostgreSQLNetworkStorage {
    pub async fn get_session_validator_performance_updater_last_processed_session_id(
        &self,
    ) -> anyhow::Result<Option<u64>> {
        let row: (Option<i64>,) = sqlx::query_as(
            r#"
                SELECT MAX(session_index)
                FROM sub_session_validator_performance
                "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(row.0.map(|index| index as u64))
    }
}

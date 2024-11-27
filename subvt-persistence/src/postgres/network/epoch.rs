use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::Epoch;

impl PostgreSQLNetworkStorage {
    pub async fn get_current_epoch(&self) -> anyhow::Result<Option<Epoch>> {
        let maybe_db_epoch: Option<(i64, i64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT index, era_index, start_block_number, start_timestamp, end_timestamp
            FROM sub_epoch
            ORDER BY index DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_db_epoch.map(|db_epoch| Epoch {
            index: db_epoch.0 as u64,
            era_index: db_epoch.1 as u32,
            start_block_number: db_epoch.2 as u32,
            start_timestamp: db_epoch.3 as u64,
            end_timestamp: db_epoch.4 as u64,
        }))
    }

    pub async fn get_epoch_by_index(&self, index: u64) -> anyhow::Result<Option<Epoch>> {
        let maybe_db_epoch: Option<(i64, i64, i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT index, era_index, start_block_number, start_timestamp, end_timestamp
            FROM sub_epoch
            WHERE index = $1
            "#,
        )
        .bind(index as i64)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_db_epoch.map(|db_epoch| Epoch {
            index: db_epoch.0 as u64,
            era_index: db_epoch.1 as u32,
            start_block_number: db_epoch.2 as u32,
            start_timestamp: db_epoch.3 as u64,
            end_timestamp: db_epoch.4 as u64,
        }))
    }
}

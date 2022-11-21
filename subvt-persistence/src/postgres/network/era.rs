use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::Era;

impl PostgreSQLNetworkStorage {
    pub async fn get_current_era(&self) -> anyhow::Result<Option<Era>> {
        let maybe_db_era: Option<(i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT index, start_timestamp, end_timestamp
            FROM sub_era
            ORDER BY index DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_db_era.map(|db_era| Era {
            index: db_era.0 as u32,
            start_timestamp: db_era.1 as u64,
            end_timestamp: db_era.2 as u64,
        }))
    }

    pub async fn get_all_eras(&self) -> anyhow::Result<Vec<Era>> {
        let db_eras: Vec<(i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT index, start_timestamp, end_timestamp
            FROM sub_era
            ORDER BY index DESC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(db_eras
            .iter()
            .map(|db_era| Era {
                index: db_era.0 as u32,
                start_timestamp: db_era.1 as u64,
                end_timestamp: db_era.2 as u64,
            })
            .collect())
    }
}

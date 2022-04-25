use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::app_event;

impl PostgreSQLNetworkStorage {
    pub async fn save_lost_nomination_event(
        &self,
        event: &app_event::LostNomination,
    ) -> anyhow::Result<u32> {
        self.save_account(&event.validator_account_id).await?;
        self.save_account(&event.nominator_stash_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_lost_nomination (validator_account_id, discovered_block_number, nominator_stash_account_id, active_amount, total_amount, nominee_count, is_onekv)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
            .bind(event.validator_account_id.to_string())
            .bind(event.discovered_block_number as i64)
            .bind(event.nominator_stash_account_id.to_string())
            .bind(event.active_amount.to_string())
            .bind(event.total_amount.to_string())
            .bind(event.nominee_count as i64)
            .bind(event.is_onekv)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }
}

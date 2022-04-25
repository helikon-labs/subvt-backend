use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_onekv_binary_version_change_event(
        &self,
        validator_account_id: &AccountId,
        prev_version: &Option<String>,
        current_version: &Option<String>,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_onekv_binary_version_change (validator_account_id, prev_version, current_version)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
            .bind(validator_account_id.to_string())
            .bind(prev_version)
            .bind(current_version)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }

    pub async fn save_onekv_rank_change_event(
        &self,
        validator_account_id: &AccountId,
        prev_rank: Option<u64>,
        current_rank: Option<u64>,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_onekv_rank_change (validator_account_id, prev_rank, current_rank)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
            .bind(validator_account_id.to_string())
            .bind(prev_rank.map(|rank| rank as i64))
            .bind(current_rank.map(|rank| rank as i64))
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }

    pub async fn save_onekv_location_change_event(
        &self,
        validator_account_id: &AccountId,
        prev_location: &Option<String>,
        current_location: &Option<String>,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_onekv_location_change (validator_account_id, prev_location, current_location)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
            .bind(validator_account_id.to_string())
            .bind(prev_location)
            .bind(current_location)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }

    pub async fn save_onekv_validity_change_event(
        &self,
        validator_account_id: &AccountId,
        is_valid: bool,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_onekv_validity_change (validator_account_id, is_valid)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(validator_account_id.to_string())
        .bind(is_valid)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0 as u32)
    }
}

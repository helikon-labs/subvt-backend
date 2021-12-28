use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::{app::app_event, crypto::AccountId};

impl PostgreSQLNetworkStorage {
    pub async fn save_new_validator(
        &self,
        validator_account_id: &AccountId,
        discovered_block_number: u64,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_new_validator (validator_account_id, discovered_block_number)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(validator_account_id.to_string())
        .bind(discovered_block_number as i64)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0 as u32)
    }

    pub async fn save_removed_validator(
        &self,
        validator_account_id: &AccountId,
        discovered_block_number: u64,
    ) -> anyhow::Result<u32> {
        self.save_account(validator_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_removed_validator (validator_account_id, discovered_block_number)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(validator_account_id.to_string())
        .bind(discovered_block_number as i64)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0 as u32)
    }

    pub async fn save_new_nomination(
        &self,
        event: &app_event::NewNomination,
    ) -> anyhow::Result<u32> {
        self.save_account(&event.validator_account_id).await?;
        self.save_account(&event.nominator_stash_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_new_nomination (validator_account_id, dicovered_block_number, nominator_stash_account_id, active_amount, total_amount, nominee_count)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(event.validator_account_id.to_string())
            .bind(event.discovered_block_number as i64)
            .bind(event.nominator_stash_account_id.to_string())
            .bind(event.active_amount.to_string())
            .bind(event.total_amount.to_string())
            .bind(event.nominee_count as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }

    pub async fn save_lost_nomination(
        &self,
        event: &app_event::LostNomination,
    ) -> anyhow::Result<u32> {
        self.save_account(&event.validator_account_id).await?;
        self.save_account(&event.nominator_stash_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_lost_nomination (validator_account_id, dicovered_block_number, nominator_stash_account_id, active_amount, total_amount, nominee_count)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(event.validator_account_id.to_string())
            .bind(event.discovered_block_number as i64)
            .bind(event.nominator_stash_account_id.to_string())
            .bind(event.active_amount.to_string())
            .bind(event.total_amount.to_string())
            .bind(event.nominee_count as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }

    pub async fn save_nomination_amount_change(
        &self,
        event: &app_event::NominationAmountChange,
    ) -> anyhow::Result<u32> {
        self.save_account(&event.validator_account_id).await?;
        self.save_account(&event.nominator_stash_account_id).await?;
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_app_event_nomination_amount_change (validator_account_id, dicovered_block_number, nominator_stash_account_id, prev_active_amount, prev_total_amount, prev_nominee_count, active_amount, total_amount, nominee_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
            .bind(event.validator_account_id.to_string())
            .bind(event.discovered_block_number as i64)
            .bind(event.nominator_stash_account_id.to_string())
            .bind(event.prev_active_amount.to_string())
            .bind(event.prev_total_amount.to_string())
            .bind(event.prev_nominee_count as i64)
            .bind(event.active_amount.to_string())
            .bind(event.total_amount.to_string())
            .bind(event.nominee_count as i64)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }
}

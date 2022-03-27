//! Persistence of Substrate democracy events.
use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Balance;

pub(crate) mod cancelled;
pub(crate) mod delegated;

impl PostgreSQLNetworkStorage {
    pub async fn save_democracy_not_passed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        referendum_index: u32,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_not_passed (block_hash, extrinsic_index, event_index, referendum_index)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(referendum_index as i64)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_democracy_passed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        referendum_index: u32,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_passed (block_hash, extrinsic_index, event_index, referendum_index)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(referendum_index as i64)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_democracy_started_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        referendum_index: u32,
        vote_threshold: &str,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_started (block_hash, extrinsic_index, event_index, referendum_index, vote_threshold)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(referendum_index as i64)
            .bind(vote_threshold)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_democracy_proposed_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        proposal_index: u32,
        deposit: Balance,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_proposed (block_hash, extrinsic_index, event_index, proposal_index, deposit)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(proposal_index as i64)
            .bind(deposit.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_democracy_seconded_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
        proposal_index: u32,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_seconded (block_hash, extrinsic_index, event_index, account_id, proposal_index)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(account_id.to_string())
            .bind(proposal_index as i64)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_democracy_undelegated_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_undelegated (block_hash, extrinsic_index, event_index, account_id)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(account_id.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn save_democracy_voted_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
        referendum_index: u32,
        vote_encoded_hex: &str,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_voted (block_hash, extrinsic_index, event_index, account_id, referendum_index, vote_encoded_hex)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(account_id.to_string())
            .bind(referendum_index as i64)
            .bind(vote_encoded_hex)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }
}

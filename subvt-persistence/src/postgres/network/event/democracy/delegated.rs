use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::app::event::democracy::DemocracyDelegatedEvent;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn save_democracy_delegated_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        original_account_id: &AccountId,
        delegate_account_id: &AccountId,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(original_account_id).await?;
        self.save_account(delegate_account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_delegated (block_hash, extrinsic_index, event_index, original_account_id, delegate_account_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(block_hash, event_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(original_account_id.to_string())
            .bind(delegate_account_id.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    pub async fn get_democracy_delegated_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<DemocracyDelegatedEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, String, String)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, original_account_id, delegate_account_id
            FROM sub_event_democracy_delegated
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(DemocracyDelegatedEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                extrinsic_index: db_event.2.map(|index| index as u32),
                event_index: db_event.3 as u32,
                original_account_id: AccountId::from_str(&db_event.4)?,
                delegate_account_id: AccountId::from_str(&db_event.5)?,
            })
        }
        Ok(events)
    }

    pub async fn update_democracy_delegated_event_nesting_index(
        &self,
        block_hash: &str,
        maybe_nesting_index: &Option<String>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_democracy_delegated
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

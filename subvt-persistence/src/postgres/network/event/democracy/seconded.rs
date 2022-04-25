use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::app::event::democracy::DemocracySecondedEvent;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
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
            ON CONFLICT(block_hash, event_index) DO NOTHING
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

    pub async fn get_democracy_seconded_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<DemocracySecondedEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, String, i64)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, account_id, proposal_index
            FROM sub_event_democracy_seconded
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
        .bind(block_hash)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(DemocracySecondedEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                extrinsic_index: db_event.2.map(|index| index as u32),
                event_index: db_event.3 as u32,
                account_id: AccountId::from_str(&db_event.4)?,
                proposal_index: db_event.5 as u64,
            })
        }
        Ok(events)
    }
}

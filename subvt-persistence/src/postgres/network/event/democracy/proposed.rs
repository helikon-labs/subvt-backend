use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::event::democracy::DemocracyProposedEvent;
use subvt_types::substrate::Balance;

impl PostgreSQLNetworkStorage {
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
            ON CONFLICT(block_hash, event_index) DO NOTHING
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

    pub async fn get_democracy_proposed_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<DemocracyProposedEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, i64, String)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, proposal_index, deposit
            FROM sub_event_democracy_proposed
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
        .bind(block_hash)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(DemocracyProposedEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                extrinsic_index: db_event.2.map(|index| index as u32),
                event_index: db_event.3 as u32,
                proposal_index: db_event.4 as u64,
                deposit: db_event.5.parse()?,
            })
        }
        Ok(events)
    }

    pub async fn update_democracy_proposed_event_batch_index(
        &self,
        block_hash: &str,
        batch_index: &Option<String>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_democracy_proposed
            SET batch_index = $1
            WHERE block_hash = $2 AND event_index = $3
            "#,
        )
        .bind(batch_index)
        .bind(block_hash)
        .bind(event_index)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}

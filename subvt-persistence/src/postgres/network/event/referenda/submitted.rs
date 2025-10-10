use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::event::referenda::ReferendumSubmittedEvent;

impl PostgreSQLNetworkStorage {
    pub async fn save_referendum_submitted_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        referendum_index: u32,
        track_id: u16,
    ) -> anyhow::Result<Option<i32>> {
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_referendum_submitted (block_hash, extrinsic_index, event_index, referendum_index, track_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(block_hash, event_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(referendum_index as i32)
            .bind(track_id as i32)
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::type_complexity)]
    pub async fn get_referendum_submitted_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ReferendumSubmittedEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, i32, i32)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, referendum_index, track_id
            FROM sub_event_referendum_submitted
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
        .bind(block_hash)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(ReferendumSubmittedEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                extrinsic_index: db_event.2.map(|index| index as u32),
                event_index: db_event.3 as u32,
                referendum_index: db_event.4 as u32,
                track_id: db_event.5 as u16,
            })
        }
        Ok(events)
    }

    pub async fn update_referendum_submitted_event_nesting_index(
        &self,
        block_hash: &str,
        maybe_nesting_index: &Option<String>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_referendum_submitted
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

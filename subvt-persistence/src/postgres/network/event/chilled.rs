use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::app::event::ChilledEvent;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn get_chilled_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ChilledEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, String)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, stash_account_id
            FROM sub_event_chilled
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
        .bind(block_hash)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(ChilledEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                extrinsic_index: db_event.2.map(|index| index as u32),
                event_index: db_event.3 as u32,
                stash_account_id: AccountId::from_str(&db_event.4)?,
            })
        }
        Ok(events)
    }

    pub async fn save_chilled_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        stash_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        self.save_account(stash_account_id).await?;
        sqlx::query(
            r#"
            INSERT INTO sub_event_chilled (block_hash, extrinsic_index, event_index, stash_account_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (block_hash, event_index, stash_account_id) DO NOTHING
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(stash_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

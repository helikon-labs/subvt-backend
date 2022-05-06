use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::app::event::democracy::DemocracyVotedEvent;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Balance;

impl PostgreSQLNetworkStorage {
    #[allow(clippy::too_many_arguments)]
    pub async fn save_democracy_voted_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        account_id: &AccountId,
        referendum_index: u32,
        aye_balance: Option<Balance>,
        nay_balance: Option<Balance>,
        conviction: Option<u8>,
    ) -> anyhow::Result<Option<i32>> {
        self.save_account(account_id).await?;
        let maybe_result: Option<(i32,)> = sqlx::query_as(
            r#"
            INSERT INTO sub_event_democracy_voted (block_hash, extrinsic_index, event_index, account_id, referendum_index, aye_balance, nay_balance, conviction)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(block_hash, event_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(account_id.to_string())
            .bind(referendum_index as i64)
            .bind(aye_balance.map(|balance| balance.to_string()))
            .bind(nay_balance.map(|balance| balance.to_string()))
            .bind(conviction.map(|c| c as i32))
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(result) = maybe_result {
            Ok(Some(result.0))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::type_complexity)]
    pub async fn get_democracy_voted_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<DemocracyVotedEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, String, i64, Option<String>, Option<String>, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, account_id, referendum_index, aye_balance, nay_balance, conviction
            FROM sub_event_democracy_voted
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(DemocracyVotedEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                extrinsic_index: db_event.2.map(|index| index as u32),
                event_index: db_event.3 as u32,
                account_id: AccountId::from_str(&db_event.4)?,
                referendum_index: db_event.5 as u64,
                aye_balance: if let Some(balance) = db_event.6 {
                    Some(balance.parse()?)
                } else {
                    None
                },
                nay_balance: if let Some(balance) = db_event.7 {
                    Some(balance.parse()?)
                } else {
                    None
                },
                conviction: db_event.8.map(|c| c as u8),
            })
        }
        Ok(events)
    }

    pub async fn update_democracy_voted_event_nesting_index(
        &self,
        block_hash: &str,
        maybe_nesting_index: &Option<String>,
        event_index: i32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_event_democracy_voted
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

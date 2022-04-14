use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::app::event::democracy::DemocracyVotedEvent;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    #[allow(clippy::type_complexity)]
    pub async fn get_chat_validator_votes_for_referendum(
        &self,
        telegram_chat_id: i64,
        referendum_id: u32,
    ) -> anyhow::Result<Vec<DemocracyVotedEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, i32, String, i64, Option<String>, Option<String>, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, account_id, referendum_index, aye_balance, nay_balance, conviction FROM (
                SELECT "id", block_hash, extrinsic_index, event_index, account_id, referendum_index, aye_balance, nay_balance, conviction, ROW_NUMBER() OVER(PARTITION BY account_id ORDER BY referendum_index) AS row
                FROM sub_event_democracy_voted
                WHERE referendum_index = $1
                AND account_id IN (
                    SELECT account_id
                    FROM sub_telegram_chat_validator
                    WHERE telegram_chat_id = $2
                    AND deleted_at IS NULL
                )
                ORDER BY "id" DESC
            ) AS v
            WHERE v.row = 1
            "#,
        )
        .bind(referendum_id as i64)
        .bind(telegram_chat_id)
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
}

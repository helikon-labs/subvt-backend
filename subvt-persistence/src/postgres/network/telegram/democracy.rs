use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::db::PostgresDemocracyVotedEvent;
use subvt_types::app::event::democracy::DemocracyVotedEvent;
use subvt_types::crypto::AccountId;

impl PostgreSQLNetworkStorage {
    pub async fn get_account_vote_for_referendum(
        &self,
        account_id: &AccountId,
        referendum_index: u32,
    ) -> anyhow::Result<Option<DemocracyVotedEvent>> {
        let maybe_db_event: Option<PostgresDemocracyVotedEvent> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, extrinsic_index, event_index, account_id, referendum_index, aye_balance, nay_balance, conviction FROM (
                SELECT "id", block_hash, extrinsic_index, event_index, account_id, referendum_index, aye_balance, nay_balance, conviction, ROW_NUMBER() OVER(PARTITION BY account_id ORDER BY referendum_index) AS row
                FROM sub_event_democracy_voted
                WHERE referendum_index = $1
                AND account_id = $2
                ORDER BY "id" DESC
            ) AS v
            WHERE v.row = 1
            "#,
        )
            .bind(referendum_index as i64)
            .bind(account_id.to_string())
            .fetch_optional(&self.connection_pool)
            .await?;
        if let Some(db_event) = maybe_db_event {
            Ok(Some(DemocracyVotedEvent::from(db_event)?))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::type_complexity)]
    pub async fn get_chat_validator_votes_for_referendum(
        &self,
        telegram_chat_id: i64,
        referendum_index: u32,
    ) -> anyhow::Result<Vec<DemocracyVotedEvent>> {
        let db_events: Vec<PostgresDemocracyVotedEvent> = sqlx::query_as(
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
        .bind(referendum_index as i64)
        .bind(telegram_chat_id)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(DemocracyVotedEvent::from(db_event)?)
        }
        Ok(events)
    }
}

use crate::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;
use subvt_types::report::{BlockSummary, HeartbeatEvent};

impl PostgreSQLNetworkStorage {
    pub async fn get_validator_session_heartbeat_event(
        &self,
        validator_account_id: &AccountId,
        session_index: u64,
    ) -> anyhow::Result<Option<HeartbeatEvent>> {
        let event: Option<(i64, String, i64, i32, String)> = sqlx::query_as(
            r#"
            SELECT B.number, B.hash, B.timestamp, E.event_index, E.im_online_key
            FROM sub_event_heartbeat_received E
            INNER JOIN sub_block B
                ON B.hash = E.block_hash
            WHERE E.validator_account_id = $1
            AND E.session_index = $2
            LIMIT 1
            "#,
        )
        .bind(validator_account_id.to_string())
        .bind(session_index as i64)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(event.map(|event| HeartbeatEvent {
            block: BlockSummary {
                number: event.0 as u64,
                hash: event.1,
                timestamp: event.2 as u64,
            },
            event_index: event.3 as u32,
            im_online_key: event.4,
        }))
    }

    pub async fn save_validator_heartbeart_event(
        &self,
        block_hash: &str,
        extrinsic_index: Option<i32>,
        event_index: i32,
        session_index: i64,
        im_online_key_hex_string: &str,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<()> {
        self.save_account(validator_account_id).await?;
        sqlx::query(
            r#"
            INSERT INTO sub_event_heartbeat_received (block_hash, extrinsic_index, event_index, session_index, im_online_key, validator_account_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (block_hash, event_index, validator_account_id) DO NOTHING
            "#,
        )
            .bind(block_hash)
            .bind(extrinsic_index)
            .bind(event_index)
            .bind(session_index)
            .bind(im_online_key_hex_string)
            .bind(validator_account_id.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

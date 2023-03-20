use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::app::event::ValidatorOfflineEvent;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::IdentificationTuple;

impl PostgreSQLNetworkStorage {
    pub async fn get_validator_offline_events_in_block(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Vec<ValidatorOfflineEvent>> {
        let db_events: Vec<(i32, String, Option<i32>, String)> = sqlx::query_as(
            r#"
            SELECT "id", block_hash, event_index, validator_account_id
            FROM sub_event_validator_offline
            WHERE block_hash = $1
            ORDER BY "id" ASC
            "#,
        )
        .bind(block_hash)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut events = Vec::new();
        for db_event in db_events {
            events.push(ValidatorOfflineEvent {
                id: db_event.0 as u32,
                block_hash: db_event.1.clone(),
                event_index: db_event.2.map(|index| index as u32),
                validator_account_id: AccountId::from_str(&db_event.3)?,
            })
        }
        Ok(events)
    }

    pub async fn save_validators_offline_event(
        &self,
        block_hash: &str,
        event_index: i32,
        identification_tuples: &[IdentificationTuple],
    ) -> anyhow::Result<()> {
        for identification_tuple in identification_tuples {
            self.save_account(&identification_tuple.0).await?;
            sqlx::query(
                r#"
                INSERT INTO sub_event_validator_offline (block_hash, event_index, validator_account_id)
                VALUES ($1, $2, $3)
                ON CONFLICT(block_hash, event_index, validator_account_id) DO NOTHING
                "#,
            )
                .bind(block_hash)
                .bind(event_index)
                .bind(identification_tuple.0.to_string())
                .execute(&self.connection_pool)
                .await?;
        }
        Ok(())
    }
}

//! PostgreSQL notifications support, and SubVT notification generator state storage.
use crate::postgres::network::PostgreSQLNetworkStorage;
use serde::Serialize;
use sqlx::postgres::PgListener;
use std::future::Future;
use subvt_types::rdb::BlockProcessedNotification;

enum Channel {
    BlockProcessed,
}

impl Channel {
    pub fn get_name(&self) -> &str {
        match self {
            Self::BlockProcessed => "block_processed",
        }
    }
}

impl PostgreSQLNetworkStorage {
    async fn notify<T>(&self, channel: &str, payload: &T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        sqlx::query(
            r#"
                SELECT pg_notify($1, $2)
                FROM (VALUES ($1, $2))
                NOTIFIES (channel, payload)
                "#,
        )
        .bind(channel)
        .bind(serde_json::to_string(payload)?)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn notify_block_processed(
        &self,
        block_number: u64,
        block_hash: &str,
    ) -> anyhow::Result<()> {
        self.notify(
            Channel::BlockProcessed.get_name(),
            &BlockProcessedNotification {
                block_number,
                block_hash: block_hash.to_string(),
            },
        )
        .await
    }

    pub async fn subscribe_to_processed_blocks<F>(
        &self,
        callback: impl Fn(BlockProcessedNotification) -> F,
    ) where
        F: Future<Output = anyhow::Result<()>>,
    {
        let mut listener = match PgListener::connect(&self.uri).await {
            Ok(listener) => listener,
            Err(error) => {
                log::error!("Error while getting Postgres listener: {error:?}");
                return;
            }
        };
        if let Err(error) = listener.listen(Channel::BlockProcessed.get_name()).await {
            log::error!("Error while trying to listen to Postgres: {error:?}");
            return;
        }
        loop {
            let pg_notification = match listener.recv().await {
                Ok(pg_notification) => pg_notification,
                Err(error) => {
                    log::error!("Error while getting Postgres notification: {error:?}");
                    return;
                }
            };
            let notification: BlockProcessedNotification =
                match serde_json::from_str(pg_notification.payload()) {
                    Ok(notification) => notification,
                    Err(error) => {
                        log::error!(
                            "Error while deserializing Postgres notification JSON: {error:?}",
                        );
                        break;
                    }
                };
            if let Err(error) = callback(notification).await {
                log::error!("Error in callback: {error:?}");
                break;
            }
        }
    }

    pub async fn get_notification_generator_state(&self) -> anyhow::Result<Option<(String, u64)>> {
        Ok(sqlx::query_as(
            r#"
                SELECT last_processed_block_hash, last_processed_block_number
                FROM sub_notification_generator_state
                "#,
        )
        .fetch_optional(&self.connection_pool)
        .await?
        .map(|state: (String, i64)| (state.0, state.1 as u64)))
    }

    pub async fn save_notification_generator_state(
        &self,
        last_processed_block_hash: &str,
        last_processed_block_number: u64,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_notification_generator_state(id, last_processed_block_hash, last_processed_block_number)
            VALUES (1, $1, $2)
            ON CONFLICT(id) DO UPDATE
            SET last_processed_block_hash = EXCLUDED.last_processed_block_hash, last_processed_block_number = EXCLUDED.last_processed_block_number, updated_at = now()
            "#,
        )
            .bind(last_processed_block_hash)
            .bind(last_processed_block_number as i64)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn notification_generator_has_processed_era(
        &self,
        era_index: u32,
    ) -> anyhow::Result<bool> {
        let result: (bool,) = sqlx::query_as(
            r#"
                SELECT EXISTS(
                    SELECT era_index
                    FROM sub_notification_generator_processed_era
                    WHERE era_index = $1
                )
                "#,
        )
        .bind(era_index as i64)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0)
    }

    pub async fn save_notification_generator_processed_era(
        &self,
        era_index: u32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_notification_generator_processed_era(era_index)
            VALUES ($1)
            ON CONFLICT(era_index) DO NOTHING
            "#,
        )
        .bind(era_index as i64)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}

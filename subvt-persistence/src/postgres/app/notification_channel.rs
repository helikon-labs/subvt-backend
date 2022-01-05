//! Storage related to the supported notification channels (email, APNS, FCM, SMS, etc.).
use crate::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::NotificationChannel;

impl PostgreSQLAppStorage {
    pub async fn get_notification_channels(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let networks: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT code
            FROM app_notification_channel
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(networks
            .iter()
            .cloned()
            .map(|db_notification_channel| NotificationChannel {
                code: db_notification_channel.0,
            })
            .collect())
    }

    pub async fn notification_channel_exists(&self, code: &str) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT code) FROM app_notification_channel
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }
}

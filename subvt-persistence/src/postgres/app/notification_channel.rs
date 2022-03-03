//! Storage related to the supported notification channels (email, APNS, FCM, SMS, etc.).
use crate::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::NotificationChannel;

impl PostgreSQLAppStorage {
    pub async fn get_notification_channels(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let channel_codes: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT code
            FROM app_notification_channel
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(channel_codes.iter().map(|c| c.0.as_str().into()).collect())
    }

    pub async fn notification_channel_exists(
        &self,
        channel: &NotificationChannel,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT code) FROM app_notification_channel
            WHERE code = $1
            "#,
        )
        .bind(channel.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }
}

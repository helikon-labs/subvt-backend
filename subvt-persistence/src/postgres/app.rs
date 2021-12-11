use crate::postgres::PostgreSQLStorage;
use subvt_types::app::{
    Network, NotificationChannel, NotificationType, User, UserNotificationChannel,
};

type PostgresNetwork = (
    i32,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl PostgreSQLStorage {
    pub async fn get_networks(&self) -> anyhow::Result<Vec<Network>> {
        let networks: Vec<PostgresNetwork> = sqlx::query_as(
            r#"
            SELECT id, hash, name, live_network_status_service_url, report_service_url, validator_details_service_url, validator_list_service_url
            FROM app_network
            ORDER BY id ASC
            "#
        )
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(networks
            .iter()
            .cloned()
            .map(|db_network| Network {
                id: db_network.0 as u32,
                hash: db_network.1,
                name: db_network.2,
                live_network_status_service_url: db_network.3,
                report_service_url: db_network.4,
                validator_details_service_url: db_network.5,
                validator_list_service_url: db_network.6,
            })
            .collect())
    }

    pub async fn save_user(&self, user: &User) -> anyhow::Result<u32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO app_user (public_key_hex)
            VALUES ($1)
            RETURNING id
            "#,
        )
        .bind(&user.public_key_hex)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0 as u32)
    }

    pub async fn user_exists_with_public_key(&self, public_key_hex: &str) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user
            WHERE public_key_hex = $1
            "#,
        )
        .bind(public_key_hex)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn user_exists_with_id(&self, id: u32) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn get_notification_channels(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let networks: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT name
            FROM app_notification_channel
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(networks
            .iter()
            .cloned()
            .map(|db_notification_channel| NotificationChannel {
                name: db_notification_channel.0,
            })
            .collect())
    }

    pub async fn notification_channel_exists(&self, name: &str) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT name) FROM app_notification_channel
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn get_notification_types(&self) -> anyhow::Result<Vec<NotificationType>> {
        let networks: Vec<(i32, String)> = sqlx::query_as(
            r#"
            SELECT id, code
            FROM app_notification_type
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(networks
            .iter()
            .cloned()
            .map(|db_notification_type| NotificationType {
                id: db_notification_type.0 as u32,
                code: db_notification_type.1,
            })
            .collect())
    }

    pub async fn get_user_notification_channels(
        &self,
        user_id: u32,
    ) -> anyhow::Result<Vec<UserNotificationChannel>> {
        let db_user_notification_channels: Vec<(i32, i32, String, String)> = sqlx::query_as(
            r#"
            SELECT id, user_id, notification_channel_name, target
            FROM app_user_notification_channel
            WHERE user_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(user_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(db_user_notification_channels
            .iter()
            .map(|db_user_notification_channel| UserNotificationChannel {
                id: db_user_notification_channel.0 as u32,
                user_id: db_user_notification_channel.1 as u32,
                channel_name: db_user_notification_channel.2.clone(),
                target: db_user_notification_channel.3.clone(),
            })
            .collect())
    }

    pub async fn user_notification_channel_exists(
        &self,
        user_id: u32,
        channel_id: u32,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user_notification_channel
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(channel_id as i32)
        .bind(user_id as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn user_notification_channel_target_exists(
        &self,
        user_notification_channel: &UserNotificationChannel,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user_notification_channel
            WHERE user_id = $1 AND notification_channel_name = $2 AND target = $3
            "#,
        )
        .bind(user_notification_channel.user_id as i32)
        .bind(&user_notification_channel.channel_name)
        .bind(&user_notification_channel.target)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn save_user_notification_channel(
        &self,
        user_notification_channel: &UserNotificationChannel,
    ) -> anyhow::Result<u32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO app_user_notification_channel (user_id, notification_channel_name, target)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(user_notification_channel.user_id as i32)
        .bind(&user_notification_channel.channel_name)
        .bind(&user_notification_channel.target)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0 as u32)
    }

    pub async fn delete_user_notification_channel(&self, id: u32) -> anyhow::Result<bool> {
        let maybe_id: Option<(i32,)> = sqlx::query_as(
            r#"
            DELETE FROM app_user_notification_channel
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_id.is_some() && maybe_id.unwrap().0 == id as i32)
    }
}

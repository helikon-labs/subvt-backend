use crate::postgres::PostgreSQLStorage;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use subvt_types::app::{
    Network, NotificationChannel, NotificationParamType, NotificationType, User,
    UserNotificationChannel, UserValidator,
};
use subvt_types::crypto::AccountId;

type PostgresNetwork = (
    i32,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

#[derive(sqlx::Type)]
#[sqlx(
    type_name = "app_notification_type_param_data_type",
    rename_all = "lowercase"
)]
enum NotificationParamDataType {
    String,
    Integer,
    Balance,
    Float,
    Boolean,
}

type PostgresNotificationParamType = (
    i32,
    String,
    String,
    i16,
    NotificationParamDataType,
    Option<String>,
    Option<String>,
    bool,
);

impl Display for NotificationParamDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NotificationParamDataType::String => "string",
                NotificationParamDataType::Integer => "integer",
                NotificationParamDataType::Balance => "balance",
                NotificationParamDataType::Float => "float",
                NotificationParamDataType::Boolean => "boolean",
            }
        )
    }
}

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

    pub async fn get_notification_types(&self) -> anyhow::Result<Vec<NotificationType>> {
        let db_notification_types: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT code
            FROM app_notification_type
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        let mut notification_types: Vec<NotificationType> = db_notification_types
            .iter()
            .cloned()
            .map(|db_notification_type| NotificationType {
                code: db_notification_type.0,
                param_types: Vec::new(),
            })
            .collect();
        // get params for each notification type
        for notification_type in notification_types.iter_mut() {
            let db_notification_param_types: Vec<PostgresNotificationParamType> = sqlx::query_as(
                r#"
                SELECT id, notification_type_code, code, "order", type, "min", "max", is_optional
                FROM app_notification_param_type
                WHERE notification_type_code = $1
                ORDER BY notification_type_code ASC, "order" ASC
                "#,
            )
            .bind(&notification_type.code)
            .fetch_all(&self.connection_pool)
            .await?;
            notification_type.param_types = db_notification_param_types
                .iter()
                .map(|db_notification_param_type| NotificationParamType {
                    id: db_notification_param_type.0 as u32,
                    notification_type_code: db_notification_param_type.1.clone(),
                    code: db_notification_param_type.2.clone(),
                    order: db_notification_param_type.3 as u8,
                    type_: db_notification_param_type.4.to_string(),
                    min: db_notification_param_type.5.clone(),
                    max: db_notification_param_type.6.clone(),
                    is_optional: db_notification_param_type.7,
                })
                .collect();
        }
        Ok(notification_types)
    }

    pub async fn get_user_notification_channels(
        &self,
        user_id: u32,
    ) -> anyhow::Result<Vec<UserNotificationChannel>> {
        let db_user_notification_channels: Vec<(i32, i32, String, String)> = sqlx::query_as(
            r#"
            SELECT id, user_id, notification_channel_code, target
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
                channel_code: db_user_notification_channel.2.clone(),
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
            WHERE user_id = $1 AND notification_channel_code = $2 AND target = $3
            "#,
        )
        .bind(user_notification_channel.user_id as i32)
        .bind(&user_notification_channel.channel_code)
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
            INSERT INTO app_user_notification_channel (user_id, notification_channel_code, target)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(user_notification_channel.user_id as i32)
        .bind(&user_notification_channel.channel_code)
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

    pub async fn user_validator_exists_with_id(
        &self,
        user_id: u32,
        user_validator_id: u32,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user_validator
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(user_validator_id as i32)
        .bind(user_id as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn user_validator_exists(
        &self,
        user_validator: &UserValidator,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user_validator
            WHERE user_id = $1 AND network_id = $2 AND validator_account_id = $3
            "#,
        )
        .bind(user_validator.user_id as i32)
        .bind(user_validator.network_id as i32)
        .bind(user_validator.validator_account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn get_user_validators(&self, user_id: u32) -> anyhow::Result<Vec<UserValidator>> {
        let db_user_validators: Vec<(i32, i32, i32, String)> = sqlx::query_as(
            r#"
            SELECT id, user_id, network_id, validator_account_id
            FROM app_user_validator
            WHERE user_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(user_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut user_validators = Vec::new();
        for db_user_validator in db_user_validators {
            user_validators.push(UserValidator {
                id: db_user_validator.0 as u32,
                user_id: db_user_validator.1 as u32,
                network_id: db_user_validator.2 as u32,
                validator_account_id: AccountId::from_str(&db_user_validator.3)?,
            });
        }
        Ok(user_validators)
    }

    pub async fn save_user_validator(&self, user_validator: &UserValidator) -> anyhow::Result<u32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO app_user_validator (user_id, network_id, validator_account_id)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(user_validator.user_id as i32)
        .bind(user_validator.network_id as i32)
        .bind(user_validator.validator_account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(result.0 as u32)
    }

    pub async fn network_exists_with_id(&self, id: u32) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_network
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn delete_user_validator(&self, id: u32) -> anyhow::Result<bool> {
        let maybe_id: Option<(i32,)> = sqlx::query_as(
            r#"
            DELETE FROM app_user_validator
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

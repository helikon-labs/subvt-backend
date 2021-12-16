use crate::postgres::PostgreSQLStorage;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use subvt_types::app::db::{PostgresNetwork, PostgresUserValidator};
use subvt_types::app::{
    Network, NotificationChannel, NotificationParamType, NotificationType, User,
    UserNotificationChannel, UserNotificationRule, UserNotificationRuleParameter, UserValidator,
};
use subvt_types::crypto::AccountId;

type PostgresUserNotificationChannel = (i32, i32, String, String);

fn postgres_user_notification_channel_to_notification_channel(
    postgres_user_notification_channel: &PostgresUserNotificationChannel,
) -> UserNotificationChannel {
    UserNotificationChannel {
        id: postgres_user_notification_channel.0 as u32,
        user_id: postgres_user_notification_channel.1 as u32,
        channel_code: postgres_user_notification_channel.2.clone(),
        target: postgres_user_notification_channel.3.clone(),
    }
}

type PostgresUserNotificationRule = (
    i32,
    i32,
    String,
    Option<String>,
    Option<i32>,
    bool,
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
    pub async fn get_network_by_id(&self, id: u32) -> anyhow::Result<Network> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, hash, name, ss58_prefix, live_network_status_service_url, report_service_url, validator_details_service_url, validator_list_service_url
            FROM app_network
            WHERE id = $1
            "#
        )
            .bind(id as i32)
            .fetch_one(&self.connection_pool)
            .await
            .map(PostgresNetwork::into)?)
    }

    pub async fn get_networks(&self) -> anyhow::Result<Vec<Network>> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, hash, name, ss58_prefix, live_network_status_service_url, report_service_url, validator_details_service_url, validator_list_service_url
            FROM app_network
            ORDER BY id ASC
            "#
        )
            .fetch_all(&self.connection_pool)
            .await?
            .iter()
            .cloned()
            .map(PostgresNetwork::into)
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

    pub async fn user_exists_by_id(&self, id: u32) -> anyhow::Result<bool> {
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

    pub async fn get_notification_type_by_code(
        &self,
        code: &str,
    ) -> anyhow::Result<NotificationType> {
        let mut notification_type = sqlx::query_as(
            r#"
            SELECT code
            FROM app_notification_type
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_one(&self.connection_pool)
        .await
        .map(|db_notification_type: (String,)| NotificationType {
            code: db_notification_type.0,
            param_types: Vec::new(),
        })?;
        // get params
        notification_type.param_types = self
            .get_notification_parameter_types(&notification_type.code)
            .await?;
        Ok(notification_type)
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
            notification_type.param_types = self
                .get_notification_parameter_types(&notification_type.code)
                .await?;
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
            WHERE user_id = $1 AND deleted_at IS NULL
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
            WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
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
            WHERE user_id = $1 AND notification_channel_code = $2 AND target = $3 AND deleted_at IS NULL
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
            UPDATE app_user_notification_channel
            SET deleted_at = now()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_id.is_some() && maybe_id.unwrap().0 == id as i32)
    }

    pub async fn user_validator_exists_by_id(
        &self,
        user_id: u32,
        user_validator_id: u32,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user_validator
            WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
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
            WHERE user_id = $1
            AND network_id = $2
            AND validator_account_id = $3
            AND deleted_at IS NULL
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
            WHERE user_id = $1 AND deleted_at IS NULL
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

    pub async fn delete_user_validator(&self, id: u32) -> anyhow::Result<bool> {
        let maybe_id: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE app_user_validator
            SET deleted_at = now()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_id.is_some() && maybe_id.unwrap().0 == id as i32)
    }

    pub async fn network_exists_by_id(&self, id: u32) -> anyhow::Result<bool> {
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

    pub async fn notification_type_exists_by_code(&self, code: &str) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT code) FROM app_notification_type
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn parameter_exists_for_notification_type(
        &self,
        notification_type_code: &str,
        parameter_type_id: u32,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_notification_param_type
            WHERE id = $1 AND notification_type_code = $2
            "#,
        )
        .bind(parameter_type_id as i32)
        .bind(notification_type_code)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn get_notification_parameter_types(
        &self,
        notification_type_code: &str,
    ) -> anyhow::Result<Vec<NotificationParamType>> {
        let db_notification_param_types: Vec<PostgresNotificationParamType> = sqlx::query_as(
            r#"
            SELECT id, notification_type_code, code, "order", type, "min", "max", is_optional
            FROM app_notification_param_type
            WHERE notification_type_code = $1
            ORDER BY notification_type_code ASC, "order" ASC
            "#,
        )
        .bind(notification_type_code)
        .fetch_all(&self.connection_pool)
        .await?;
        let param_types: Vec<NotificationParamType> = db_notification_param_types
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
        Ok(param_types)
    }

    pub async fn get_user_notification_rule_validators(
        &self,
        rule_id: u32,
    ) -> anyhow::Result<Vec<UserValidator>> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, user_id, network_id, validator_account_id
            FROM app_user_validator
            WHERE id IN (
                SELECT user_validator_id
                FROM app_user_notification_rule_validator
                WHERE user_notification_rule_id = $1
            )
            ORDER BY id ASC
            "#,
        )
        .bind(rule_id as i32)
        .fetch_all(&self.connection_pool)
        .await?
        .iter()
        .cloned()
        .map(PostgresUserValidator::into)
        .collect())
    }

    pub async fn get_user_notification_rule_channels(
        &self,
        rule_id: u32,
    ) -> anyhow::Result<Vec<UserNotificationChannel>> {
        let db_user_notification_channels: Vec<PostgresUserNotificationChannel> = sqlx::query_as(
            r#"
            SELECT id, user_id, notification_channel_code, target
            FROM app_user_notification_channel
            WHERE id IN (
                SELECT user_notification_channel_id
                FROM app_user_notification_rule_channel
                WHERE user_notification_rule_id = $1
            )
            ORDER BY id ASC
            "#,
        )
        .bind(rule_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(db_user_notification_channels
            .iter()
            .map(postgres_user_notification_channel_to_notification_channel)
            .collect())
    }

    pub async fn get_user_notification_rule_parameters(
        &self,
        rule_id: u32,
    ) -> anyhow::Result<Vec<UserNotificationRuleParameter>> {
        let db_user_notification_rule_parameters: Vec<(i32, i32, i16, String)> = sqlx::query_as(
            r#"
            SELECT user_notification_rule_id, notification_param_type_id, "order", "value"
            FROM app_user_notification_rule_param
            WHERE user_notification_rule_id = $1
            ORDER BY "order" ASC
            "#,
        )
        .bind(rule_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(db_user_notification_rule_parameters
            .iter()
            .map(|input| input.into())
            .collect())
    }

    pub async fn get_user_notification_rule_by_id(
        &self,
        rule_id: u32,
    ) -> anyhow::Result<Option<UserNotificationRule>> {
        let maybe_db_notification_rule: Option<PostgresUserNotificationRule> = sqlx::query_as(
            r#"
            SELECT id, user_id, notification_type_code, name, network_id, is_for_all_validators, notes
            FROM app_user_notification_rule
            WHERE id = $1
            "#
        )
            .bind(rule_id as i32)
            .fetch_optional(&self.connection_pool)
            .await?;
        let db_notification_rule = if let Some(db_notification_rule) = maybe_db_notification_rule {
            db_notification_rule
        } else {
            return Ok(None);
        };
        // get network
        let maybe_network = if let Some(network_id) = db_notification_rule.4 {
            Some(self.get_network_by_id(network_id as u32).await?)
        } else {
            None
        };
        Ok(Some(UserNotificationRule {
            id: db_notification_rule.0 as u32,
            notification_type: self
                .get_notification_type_by_code(&db_notification_rule.2)
                .await?,
            name: db_notification_rule.3,
            network: maybe_network,
            is_for_all_validators: db_notification_rule.5,
            validators: self
                .get_user_notification_rule_validators(db_notification_rule.0 as u32)
                .await?,
            notification_channels: self
                .get_user_notification_rule_channels(db_notification_rule.0 as u32)
                .await?,
            parameters: self
                .get_user_notification_rule_parameters(db_notification_rule.0 as u32)
                .await?,
            notes: db_notification_rule.6,
        }))
    }

    pub async fn get_user_notification_rules(
        &self,
        user_id: u32,
    ) -> anyhow::Result<Vec<UserNotificationRule>> {
        let rule_ids: Vec<(i32,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM app_user_notification_rule
            WHERE user_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(user_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut rules = Vec::new();
        for rule_id in rule_ids {
            if let Some(rule) = self
                .get_user_notification_rule_by_id(rule_id.0 as u32)
                .await?
            {
                rules.push(rule);
            }
        }
        Ok(rules)
    }

    pub async fn user_notification_rule_exists_by_id(
        &self,
        user_id: u32,
        rule_id: u32,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM app_user_notification_rule
            WHERE id = $1 AND user_id = $2
            AND deleted_at IS NULL
            "#,
        )
        .bind(rule_id as i32)
        .bind(user_id as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn delete_user_notification_rule(&self, id: u32) -> anyhow::Result<bool> {
        let maybe_id: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE app_user_notification_rule
            SET deleted_at = now()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_id.is_some() && maybe_id.unwrap().0 == id as i32)
    }

    pub async fn save_user_notification_rule(
        &self,
        user_id: u32,
        notification_type_code: &str,
        (name, notes): (Option<&str>, Option<&str>),
        network_id: Option<u32>,
        is_for_all_validators: bool,
        (user_validator_ids, user_notification_channel_ids, parameters): (
            &HashSet<u32>,
            &HashSet<u32>,
            &[UserNotificationRuleParameter],
        ),
    ) -> anyhow::Result<u32> {
        let mut transaction = self.connection_pool.begin().await?;
        // insert notification rule
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO app_user_notification_rule (user_id, notification_type_code, name, network_id, is_for_all_validators, notes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(user_id as i32)
            .bind(notification_type_code)
            .bind(name)
            .bind(network_id.map(|network_id| network_id as i32))
            .bind(is_for_all_validators)
            .bind(notes)
            .fetch_one(&self.connection_pool)
            .await?;
        let user_notification_rule_id = result.0;
        // insert validators
        for user_validator_id in user_validator_ids {
            sqlx::query(
                r#"
                INSERT INTO app_user_notification_rule_validator (user_notification_rule_id, user_validator_id)
                VALUES ($1, $2)
                "#,
            )
                .bind(user_notification_rule_id)
                .bind(*user_validator_id as i32)
                .execute(&mut transaction)
                .await?;
        }
        // insert channel ids
        for user_notification_channel_id in user_notification_channel_ids {
            sqlx::query(
                r#"
                INSERT INTO app_user_notification_rule_channel (user_notification_rule_id, user_notification_channel_id)
                VALUES ($1, $2)
                "#,
            )
                .bind(user_notification_rule_id)
                .bind(*user_notification_channel_id as i32)
                .execute(&mut transaction)
                .await?;
        }
        // insert params
        for param in parameters {
            sqlx::query(
                r#"
                INSERT INTO app_user_notification_rule_param (user_notification_rule_id, notification_param_type_id, "order", value)
                VALUES ($1, $2, $3, $4)
                "#,
            )
                .bind(user_notification_rule_id)
                .bind(param.parameter_type_id as i32)
                .bind(param.order as i16)
                .bind(&param.value)
                .execute(&mut transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(user_notification_rule_id as u32)
    }
}

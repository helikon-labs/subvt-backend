//! Storage related to SubVT application users.
use crate::postgres::app::PostgreSQLAppStorage;
use std::collections::HashSet;
use std::str::FromStr;
use subvt_types::app::db::{
    PostgresUserNotificationChannel, PostgresUserNotificationRule, PostgresUserValidator,
};
use subvt_types::app::{
    NotificationPeriodType, User, UserNotificationChannel, UserNotificationRule,
    UserNotificationRuleParameter, UserValidator,
};
use subvt_types::crypto::AccountId;

impl PostgreSQLAppStorage {
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

    pub async fn user_exists_by_public_key(&self, public_key_hex: &str) -> anyhow::Result<bool> {
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

    pub async fn get_user_by_public_key(
        &self,
        public_key_hex: &str,
    ) -> anyhow::Result<Option<User>> {
        let maybe_db_user: Option<(i32, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, public_key_hex
            FROM app_user
            WHERE public_key_hex = $1
            "#,
        )
        .bind(public_key_hex)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(db_user) = maybe_db_user {
            Ok(Some(User {
                id: db_user.0 as u32,
                public_key_hex: db_user.1,
            }))
        } else {
            Ok(None)
        }
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
                channel: db_user_notification_channel.2.as_str().into(),
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
            .bind(&user_notification_channel.channel.to_string())
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
        .bind(&user_notification_channel.channel.to_string())
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

    pub async fn delete_user_validator_by_account_id(
        &self,
        user_id: u32,
        network_id: u32,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let maybe_id: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE app_user_validator
            SET deleted_at = now()
            WHERE user_id = $1 AND network_id = $2 AND validator_account_id = $3
            RETURNING id
            "#,
        )
        .bind(user_id as i32)
        .bind(network_id as i32)
        .bind(validator_account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_id.is_some())
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
            AND deleted_at IS NULL
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
        Ok(sqlx::query_as(
            r#"
            SELECT id, user_id, notification_channel_code, target
            FROM app_user_notification_channel
            WHERE id IN (
                SELECT user_notification_channel_id
                FROM app_user_notification_rule_channel
                WHERE user_notification_rule_id = $1
            )
            AND deleted_at IS NULL
            ORDER BY id ASC
            "#,
        )
        .bind(rule_id as i32)
        .fetch_all(&self.connection_pool)
        .await?
        .iter()
        .cloned()
        .map(PostgresUserNotificationChannel::into)
        .collect())
    }

    pub async fn get_user_notification_rule_parameters(
        &self,
        rule_id: u32,
    ) -> anyhow::Result<Vec<UserNotificationRuleParameter>> {
        let db_user_notification_rule_parameters: Vec<(i32, i32, String, i16, String)> = sqlx::query_as(
            r#"
            SELECT AUNRP.user_notification_rule_id, AUNRP.notification_param_type_id, ANPT.code, ANPT."order", AUNRP."value"
            FROM app_user_notification_rule_param AUNRP, app_notification_param_type ANPT
            WHERE AUNRP.notification_param_type_id = ANPT.id
            AND AUNRP.user_notification_rule_id = $1
            ORDER BY ANPT."order" ASC
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
            SELECT id, user_id, notification_type_code, name, network_id, is_for_all_validators, period_type, period, notes
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
            user_id: db_notification_rule.1 as u32,
            notification_type: self
                .get_notification_type_by_code(&db_notification_rule.2)
                .await?,
            name: db_notification_rule.3,
            network: maybe_network,
            is_for_all_validators: db_notification_rule.5,
            period_type: db_notification_rule.6,
            period: db_notification_rule.7 as u16,
            validators: self
                .get_user_notification_rule_validators(db_notification_rule.0 as u32)
                .await?,
            notification_channels: self
                .get_user_notification_rule_channels(db_notification_rule.0 as u32)
                .await?,
            parameters: self
                .get_user_notification_rule_parameters(db_notification_rule.0 as u32)
                .await?,
            notes: db_notification_rule.8,
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
        (network_id, is_for_all_validators): (Option<u32>, bool),
        (period_type, period): (&NotificationPeriodType, u16),
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
            INSERT INTO app_user_notification_rule (user_id, notification_type_code, name, network_id, is_for_all_validators, period_type, period, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
            .bind(user_id as i32)
            .bind(notification_type_code)
            .bind(name)
            .bind(network_id.map(|network_id| network_id as i32))
            .bind(is_for_all_validators)
            .bind(period_type)
            .bind(period as i32)
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
                INSERT INTO app_user_notification_rule_param (user_notification_rule_id, notification_param_type_id, value)
                VALUES ($1, $2, $3)
                "#,
            )
                .bind(user_notification_rule_id)
                .bind(param.parameter_type_id as i32)
                .bind(&param.value)
                .execute(&mut transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(user_notification_rule_id as u32)
    }
}

use crate::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::db::{PostgresNotification, PostgresNotificationParamType};
use subvt_types::app::{
    Notification, NotificationParamType, NotificationPeriodType, UserNotificationRule,
};
use subvt_types::crypto::AccountId;

impl PostgreSQLAppStorage {
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
                type_: db_notification_param_type.4.clone(),
                min: db_notification_param_type.5.clone(),
                max: db_notification_param_type.6.clone(),
                is_optional: db_notification_param_type.7,
            })
            .collect();
        Ok(param_types)
    }

    pub async fn get_notification_rules_for_validator(
        &self,
        notification_type_code: &str,
        network_id: u32,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<Vec<UserNotificationRule>> {
        let rule_ids: Vec<(i32,)> = sqlx::query_as(
            r#"
            SELECT "id"
            FROM app_user_notification_rule UNR
            WHERE UNR.notification_type_code = $1
            AND UNR.deleted_at IS NULL
            AND (UNR.network_id IS NULL OR UNR.network_id = $2)
            AND (
                (
                    UNR.is_for_all_validators = true
                    AND EXISTS (
                        SELECT DISTINCT "id"
                        FROM app_user_validator UV1
                        WHERE UV1.network_id = $2
                        AND UV1.validator_account_id = $3
                        AND UV1.deleted_at IS NULL
                    )
                )
                OR
                (
                    UNR.is_for_all_validators = false
                    AND EXISTS (
                        SELECT id FROM app_user_notification_rule_validator UNRV
                        WHERE UNRV.user_notification_rule_id = UNR.id
                        AND EXISTS(
                            SELECT DISTINCT "id"
                            FROM app_user_validator UV2
                            WHERE UV2.network_id = $2
                            AND UV2.validator_account_id = $3
                            AND UV2.id = UNRV.user_validator_id
                            AND UV2.deleted_at IS NULL
                        )
                    )
                )
            );
            "#,
        )
        .bind(notification_type_code)
        .bind(network_id as i32)
        .bind(validator_account_id.to_string())
        .fetch_all(&self.connection_pool)
        .await?;
        let mut result = Vec::new();
        for rule_id in rule_ids {
            if let Some(rule) = self
                .get_user_notification_rule_by_id(rule_id.0 as u32)
                .await?
            {
                result.push(rule);
            }
        }
        Ok(result)
    }

    pub async fn save_notification(&self, notification: &Notification) -> anyhow::Result<u32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO app_notification (user_id, user_notification_rule_id, network_id, period_type, period, validator_account_id, validator_account_json, notification_type_code, user_notification_channel_id, notification_channel_code, notification_target, data_json, log)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
            "#,
        )
            .bind(notification.user_id as i32)
            .bind(notification.user_notification_rule_id as i32)
            .bind(notification.network_id as i32)
            .bind(&notification.period_type)
            .bind(notification.period as i32)
            .bind(notification.validator_account_id.to_string())
            .bind(&notification.validator_account_json)
            .bind(&notification.notification_type_code)
            .bind(notification.user_notification_channel_id as i32)
            .bind(&notification.notification_channel_code)
            .bind(&notification.notification_target)
            .bind(&notification.data_json)
            .bind(&notification.log)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(result.0 as u32)
    }

    pub async fn get_pending_notifications_by_period_type(
        &self,
        period_type: &NotificationPeriodType,
        period: u32,
    ) -> anyhow::Result<Vec<Notification>> {
        let db_notifications: Vec<PostgresNotification> = sqlx::query_as(
            r#"
            SELECT id, user_id, user_notification_rule_id, network_id, period_type, period, validator_account_id, validator_account_json, notification_type_code, user_notification_channel_id, notification_channel_code, notification_target, data_json, log
            FROM app_notification
            WHERE processing_started_at IS NULL
            AND period_type = $1
            AND (period = 0 OR ($2 % period) = 0)
            "#,
        )
            .bind(period_type)
            .bind(period as i32)
            .fetch_all(&self.connection_pool)
            .await?;
        let mut notifications = vec![];
        for db_notification in db_notifications {
            notifications.push(Notification::from(db_notification)?);
        }
        Ok(notifications)
    }

    pub async fn reset_pending_and_failed_notifications(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE app_notification
            SET processing_started_at = NULL, failed_at = NULL
            WHERE sent_at IS NULL
            "#,
        )
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn mark_notification_processing(&self, id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE app_notification
            SET processing_started_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn mark_notification_failed(&self, id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE app_notification
            SET failed_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn mark_notification_sent(&self, id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE app_notification
            SET sent_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn mark_notification_delivered(&self, id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE app_notification
            SET delivered_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn mark_notification_read(&self, id: u32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE app_notification
            SET read_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn set_notification_log(&self, id: u32, log: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE app_notification
            SET log = $1
            WHERE id = $2
            "#,
        )
        .bind(log)
        .bind(id as i32)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}

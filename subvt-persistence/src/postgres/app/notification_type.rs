//! Notification type (authorship, offences, new/lost/changed nominations, etc.) related storage.
use crate::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::notification::NotificationType;

impl PostgreSQLAppStorage {
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
}

use crate::postgres::network::PostgreSQLNetworkStorage;

impl PostgreSQLNetworkStorage {
    pub async fn save_bug_report(
        &self,
        telegram_chat_id: i64,
        content: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_telegram_bot_bug_report (telegram_chat_id, content)
            VALUES ($1, $2)
            "#,
        )
        .bind(telegram_chat_id)
        .bind(content)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn save_feature_request(
        &self,
        telegram_chat_id: i64,
        content: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_telegram_bot_feature_request(telegram_chat_id, content)
            VALUES ($1, $2)
            "#,
        )
        .bind(telegram_chat_id)
        .bind(content)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }
}

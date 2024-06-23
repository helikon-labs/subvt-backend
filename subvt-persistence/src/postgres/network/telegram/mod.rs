//! Telegram-bot-related storage. Used by the `subvt-telegram-bot` crate.
use crate::postgres::network::PostgreSQLNetworkStorage;
use std::str::FromStr;
use subvt_types::crypto::AccountId;
use subvt_types::telegram::{TelegramChatState, TelegramChatValidator};

mod democracy;
mod report;

impl PostgreSQLNetworkStorage {
    pub async fn get_chat_count(&self) -> anyhow::Result<u64> {
        let chat_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT telegram_chat_id)
            FROM sub_telegram_chat
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(chat_count.0 as u64)
    }

    pub async fn get_chat_total_validator_count(&self) -> anyhow::Result<u64> {
        let validator_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT account_id) FROM (
                SELECT account_id
                FROM sub_telegram_chat_validator
                WHERE deleted_at IS NULL
            ) AS validators
            "#,
        )
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(validator_count.0 as u64)
    }

    pub async fn get_chat_ids(&self) -> anyhow::Result<Vec<i64>> {
        let chat_validators: Vec<(i64,)> = sqlx::query_as(
            r#"
            SELECT telegram_chat_id
            FROM sub_telegram_chat
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(chat_validators.iter().map(|v| v.0).collect())
    }

    pub async fn get_chat_validator_by_id(
        &self,
        telegram_chat_id: i64,
        chat_validator_id: u64,
    ) -> anyhow::Result<Option<TelegramChatValidator>> {
        let maybe_chat_validator: Option<(String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT account_id, address, display
            FROM sub_telegram_chat_validator
            WHERE telegram_chat_id = $1
            AND id = $2
            AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .bind(chat_validator_id as i32)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(match maybe_chat_validator {
            Some(v) => Some(TelegramChatValidator {
                id: chat_validator_id,
                account_id: AccountId::from_str(&v.0)?,
                address: v.1,
                display: v.2,
            }),
            None => None,
        })
    }

    pub async fn get_chat_validator_by_address(
        &self,
        telegram_chat_id: i64,
        address: &str,
    ) -> anyhow::Result<Option<TelegramChatValidator>> {
        let maybe_chat_validator: Option<(i32, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, account_id, display
            FROM sub_telegram_chat_validator
            WHERE telegram_chat_id = $1
            AND address = $2
            AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .bind(address)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(match maybe_chat_validator {
            Some(v) => Some(TelegramChatValidator {
                id: v.0 as u64,
                account_id: AccountId::from_str(&v.1)?,
                address: address.to_owned(),
                display: v.2,
            }),
            None => None,
        })
    }

    pub async fn get_chat_validator_by_account_id(
        &self,
        telegram_chat_id: i64,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<TelegramChatValidator>> {
        let maybe_chat_validator: Option<(i32, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, address, display
            FROM sub_telegram_chat_validator
            WHERE telegram_chat_id = $1
            AND account_id = $2
            AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_chat_validator.map(|v| TelegramChatValidator {
            id: v.0 as u64,
            account_id: *account_id,
            address: v.1,
            display: v.2,
        }))
    }

    pub async fn get_chat_validators(
        &self,
        telegram_chat_id: i64,
    ) -> anyhow::Result<Vec<TelegramChatValidator>> {
        let chat_validators: Vec<(i32, String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, account_id, address, display
            FROM sub_telegram_chat_validator
            WHERE telegram_chat_id = $1
            AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .fetch_all(&self.connection_pool)
        .await?;
        let mut result = vec![];
        for chat_validator in chat_validators {
            result.push(TelegramChatValidator {
                id: chat_validator.0 as u64,
                account_id: AccountId::from_str(&chat_validator.1)?,
                address: chat_validator.2.clone(),
                display: chat_validator.3.to_owned(),
            });
        }
        Ok(result)
    }

    pub async fn chat_exists_by_id(&self, telegram_chat_id: i64) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT telegram_chat_id) FROM sub_telegram_chat
            WHERE telegram_chat_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn chat_is_deleted(&self, telegram_chat_id: i64) -> anyhow::Result<bool> {
        let maybe_is_deleted: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT (deleted_at IS NOT NULL) AS is_deleted FROM sub_telegram_chat
            WHERE telegram_chat_id = $1
            "#,
        )
        .bind(telegram_chat_id)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(is_deleted) = maybe_is_deleted {
            Ok(is_deleted.0)
        } else {
            Ok(false)
        }
    }

    pub async fn delete_chat(&self, telegram_chat_id: i64) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_telegram_chat
            SET deleted_at = now()
            WHERE telegram_chat_id = $1
            "#,
        )
        .bind(telegram_chat_id)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn undelete_chat(&self, telegram_chat_id: i64) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_telegram_chat
            SET deleted_at = NULL
            WHERE telegram_chat_id = $1
            "#,
        )
        .bind(telegram_chat_id)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn update_chat_validator_display(
        &self,
        account_id: &AccountId,
        display: &Option<String>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_telegram_chat_validator
            SET display = $1
            WHERE account_id = $2
            "#,
        )
        .bind(display)
        .bind(account_id.to_string())
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn save_chat(
        &self,
        app_user_id: u32,
        telegram_chat_id: i64,
        state: &TelegramChatState,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO sub_telegram_chat (app_user_id, telegram_chat_id, state)
            VALUES ($1, $2, $3)
            ON CONFLICT(telegram_chat_id) DO UPDATE
            SET app_user_id = $1, deleted_at = NULL
            "#,
        )
        .bind(app_user_id as i32)
        .bind(telegram_chat_id)
        .bind(state.to_string())
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn get_chat_app_user_id(&self, telegram_chat_id: i64) -> anyhow::Result<u32> {
        let app_user_id: (i32,) = sqlx::query_as(
            r#"
            SELECT app_user_id FROM sub_telegram_chat
            WHERE telegram_chat_id = $1
            "#,
        )
        .bind(telegram_chat_id)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(app_user_id.0 as u32)
    }

    pub async fn chat_has_validator(
        &self,
        telegram_chat_id: i64,
        validator_account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT id) FROM sub_telegram_chat_validator
            WHERE telegram_chat_id = $1 AND account_id = $2 AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .bind(validator_account_id.to_string())
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub async fn add_validator_to_chat(
        &self,
        telegram_chat_id: i64,
        account_id: &AccountId,
        address: &str,
        display: &Option<String>,
    ) -> anyhow::Result<u64> {
        self.save_account(account_id).await?;
        let id: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_telegram_chat_validator (telegram_chat_id, account_id, address, display)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(telegram_chat_id, account_id) DO UPDATE SET deleted_at = NULL
            RETURNING id
            "#,
        )
        .bind(telegram_chat_id)
        .bind(account_id.to_string())
        .bind(address)
        .bind(display)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(id.0 as u64)
    }

    pub async fn remove_validator_from_chat(
        &self,
        telegram_chat_id: i64,
        account_id: &AccountId,
    ) -> anyhow::Result<bool> {
        let maybe_id: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE sub_telegram_chat_validator
            SET deleted_at = now()
            WHERE telegram_chat_id = $1
            AND account_id = $2
            RETURNING id
            "#,
        )
        .bind(telegram_chat_id)
        .bind(account_id.to_string())
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_id.is_some())
    }

    pub async fn set_chat_state(
        &self,
        telegram_chat_id: i64,
        state: TelegramChatState,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_telegram_chat
            SET state = $1
            WHERE telegram_chat_id = $2
            "#,
        )
        .bind(state.to_string())
        .bind(telegram_chat_id)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn get_chat_state(
        &self,
        telegram_chat_id: i64,
    ) -> anyhow::Result<Option<TelegramChatState>> {
        let maybe_state_str: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT state FROM sub_telegram_chat
            WHERE telegram_chat_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .fetch_optional(&self.connection_pool)
        .await?;
        if let Some(state_str) = maybe_state_str {
            let state = TelegramChatState::from_str(&state_str.0)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    pub async fn get_chat_validator_count(&self, telegram_chat_id: i64) -> anyhow::Result<u16> {
        let chat_validator_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT account_id)
            FROM sub_telegram_chat_validator
            WHERE telegram_chat_id = $1
            AND deleted_at IS NULL
            "#,
        )
        .bind(telegram_chat_id)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(chat_validator_count.0 as u16)
    }

    pub async fn set_chat_settings_message_id(
        &self,
        telegram_chat_id: i64,
        settings_message_id: Option<i32>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE sub_telegram_chat
            SET settings_message_id = $1
            WHERE telegram_chat_id = $2
            "#,
        )
        .bind(settings_message_id)
        .bind(telegram_chat_id)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    pub async fn get_chat_settings_message_id(
        &self,
        telegram_chat_id: i64,
    ) -> anyhow::Result<Option<i32>> {
        let settings_message_id: (Option<i32>,) = sqlx::query_as(
            r#"
            SELECT settings_message_id FROM sub_telegram_chat
            WHERE telegram_chat_id = $1
            "#,
        )
        .bind(telegram_chat_id)
        .fetch_one(&self.connection_pool)
        .await?;

        Ok(settings_message_id.0)
    }

    pub async fn save_chat_command_log(
        &self,
        telegram_chat_id: i64,
        command: &str,
    ) -> anyhow::Result<u64> {
        let id: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_telegram_chat_activity_log (telegram_chat_id, command)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(telegram_chat_id)
        .bind(command)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(id.0 as u64)
    }

    pub async fn save_chat_query_log(
        &self,
        telegram_chat_id: i64,
        query: &str,
    ) -> anyhow::Result<u64> {
        let id: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO sub_telegram_chat_activity_log (telegram_chat_id, query)
            VALUES ($1, $2)
            RETURNING id
            "#,
        )
        .bind(telegram_chat_id)
        .bind(query)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(id.0 as u64)
    }
}

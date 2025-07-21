//! Telegram bot. Former 1KV Telegram Bot (https://github.com/helikon-labs/polkadot-kusama-1kv-telegram-bot)
//! migrated to the SubVT backend (https://github.com/helikon-labs/subvt-backend/tree/development).
#![warn(clippy::disallowed_types)]

use crate::messenger::Messenger;
use crate::{
    messenger::{message::MessageType, MessengerImpl},
    query::Query,
};
use async_trait::async_trait;
pub use frankenstein::client_reqwest::Bot;
use frankenstein::methods::GetUpdatesParams;
pub use frankenstein::methods::SendMessageParams;
pub use frankenstein::types::{ChatId, LinkPreviewOptions};
use frankenstein::types::{ChatType, MaybeInaccessibleMessage, Message};
pub use frankenstein::{AsyncTelegramApi, ParseMode};
use lazy_static::lazy_static;
use regex::Regex;
use rustc_hash::FxHashSet as HashSet;
use std::str::FromStr;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_persistence::redis::Redis;
use subvt_service_common::Service;
use subvt_types::app::{
    notification::{
        NotificationChannel, NotificationPeriodType, NotificationTypeCode, UserNotificationChannel,
    },
    User,
};
use subvt_types::crypto::AccountId;
use subvt_types::telegram::TelegramChatState;
mod command;
pub mod messenger;
mod metrics;
mod query;
#[cfg(test)]
mod test;

lazy_static! {
    static ref CONFIG: Config = Config::default();
    static ref CMD_REGEX: Regex = Regex::new(r"^/([a-zA-Z0-9_]+[@a-zA-Z0-9_]?)(\s+[a-zA-Z0-9_-]+)*").unwrap();
    static ref CMD_ARG_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    static ref SPLITTER_REGEX: Regex = Regex::new(r"\s+").unwrap();
    /// Default notification rules.
    static ref DEFAULT_RULES: Vec<(NotificationTypeCode, NotificationPeriodType, u16)> = vec![
        (
            NotificationTypeCode::ChainValidateExtrinsic,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorActive,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorActiveNextSession,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorBlockAuthorship,
            NotificationPeriodType::Hour,
            1,
        ),
        (
            NotificationTypeCode::ChainValidatorChilled,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorIdentityChanged,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorInactive,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorInactiveNextSession,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorLostNomination,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorNewNomination,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorOfflineOffence,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorPayoutStakers,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorSessionKeysChanged,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorSetController,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorUnclaimedPayout,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorStartedParaValidating,
            NotificationPeriodType::Off,
            0,
        ),
        (
            NotificationTypeCode::ChainValidatorStoppedParaValidating,
            NotificationPeriodType::Off,
            0,
        ),
        (
            NotificationTypeCode::OneKVValidatorLocationChange,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::OneKVValidatorOnlineStatusChange,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::OneKVValidatorRankChange,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::OneKVValidatorValidityChange,
            NotificationPeriodType::Immediate,
            0,
        ),
        // referenda/opengov
        (
            NotificationTypeCode::ReferendumConfirmed,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ReferendumDecisionStarted,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ReferendumRejected,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ReferendumTimedOut,
            NotificationPeriodType::Immediate,
            0,
        ),
        (
            NotificationTypeCode::ReferendumCancelled,
            NotificationPeriodType::Immediate,
            0,
        ),
    ];
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum TelegramBotError {
    #[error("{0}")]
    Error(String),
}

pub struct TelegramBot<M: Messenger + Send + Sync> {
    /// SubVT application deployment Postgres helper.
    app_postgres: PostgreSQLAppStorage,
    /// SubVT network deployment Postgres helper.
    network_postgres: PostgreSQLNetworkStorage,
    /// SubVT application deployment Redis helper.
    redis: Redis,
    /// Telegram API helper.
    api: Bot,
    /// Telegram messenger struct.
    messenger: M,
}

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub async fn new() -> anyhow::Result<TelegramBot<MessengerImpl>> {
        let app_postgres =
            PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?;
        let network_postgres =
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
        let redis = Redis::new()?;
        let api = Bot::new(&CONFIG.telegram_bot.api_token);

        Ok(TelegramBot {
            app_postgres,
            network_postgres,
            redis,
            api,
            messenger: MessengerImpl::new()?,
        })
    }
}

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    async fn reset_chat_state(&self, telegram_chat_id: i64) -> anyhow::Result<()> {
        self.network_postgres
            .set_chat_state(telegram_chat_id, TelegramChatState::Default)
            .await?;
        Ok(())
    }

    /// Create default notification for the SubVT user corresponding to the chat.
    async fn create_default_notification_rules(
        &self,
        app_user_id: u32,
        channel_id: u32,
    ) -> anyhow::Result<()> {
        let mut channel_id_set = HashSet::default();
        channel_id_set.insert(channel_id);
        for rule in DEFAULT_RULES.iter() {
            self.app_postgres
                .save_user_notification_rule(
                    app_user_id,
                    &rule.0.to_string(),
                    (None, None),
                    (Some(CONFIG.substrate.network_id), true),
                    (&rule.1, rule.2),
                    (&HashSet::default(), &channel_id_set, &[]),
                )
                .await?;
        }
        Ok(())
    }

    /// Create a SubVT application user for the Telegram chat.
    async fn create_app_user(&self, chat_id: i64) -> anyhow::Result<u32> {
        log::info!("Create new app user, notification channel and rules for chat {chat_id}.",);
        // save app user
        let app_user_id = self.app_postgres.save_user(&User::default(), None).await?;
        // save notification channel
        let channel_id = self
            .app_postgres
            .save_user_notification_channel(&UserNotificationChannel {
                id: 0,
                user_id: app_user_id,
                channel: NotificationChannel::Telegram,
                target: chat_id.to_string(),
            })
            .await?;
        let mut channel_id_set = HashSet::default();
        channel_id_set.insert(channel_id);
        // create notification rules
        self.create_default_notification_rules(app_user_id, channel_id)
            .await?;
        Ok(app_user_id)
    }

    /// Works after each Telegram update to see if the user exists/deleted. Created/undeletes
    /// as necessary.
    async fn save_or_restore_chat(&self, chat_id: i64) -> anyhow::Result<()> {
        if !self.network_postgres.chat_exists_by_id(chat_id).await? {
            if self.network_postgres.chat_is_deleted(chat_id).await? {
                let app_user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
                self.app_postgres.undelete_user(app_user_id).await?;
                self.network_postgres.undelete_chat(chat_id).await?;
            } else {
                let app_user_id = self.create_app_user(chat_id).await?;
                log::info!("Save new chat {chat_id}. App user id {app_user_id}.");
                self.network_postgres
                    .save_chat(app_user_id, chat_id, &TelegramChatState::Default)
                    .await?;
            }
            self.update_metrics_chat_count().await?;
        }
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    async fn process_message(&self, message: &Message) -> anyhow::Result<()> {
        // group chat started - send intro
        if let Some(group_chat_created) = message.group_chat_created {
            if group_chat_created {
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        message.chat.id,
                        Box::new(MessageType::Intro),
                    )
                    .await?;
                self.update_metrics_chat_count().await?;
                return Ok(());
            }
        }
        // text message
        if let Some(text) = message.text.clone() {
            let text = text.trim();
            if CMD_REGEX.is_match(text) {
                log::info!("New command: {text}");
                self.reset_chat_state(message.chat.id).await?;
                let (command, arguments): (String, Vec<String>) = {
                    let parts: Vec<String> = SPLITTER_REGEX.split(text).map(String::from).collect();
                    (
                        parts[0].clone(),
                        parts[1..]
                            .iter()
                            .filter(|arg| CMD_ARG_REGEX.is_match(arg))
                            .cloned()
                            .collect(),
                    )
                };
                let command = command.replace(&CONFIG.telegram_bot.username, "");
                self.process_command(message.chat.id, &command, &arguments)
                    .await?;
            } else {
                log::info!("New text message: {text}");
                let maybe_state = self
                    .network_postgres
                    .get_chat_state(message.chat.id)
                    .await?;
                if let Some(state) = maybe_state {
                    match state {
                        TelegramChatState::AddValidator => {
                            if AccountId::from_str(text).is_ok() {
                                self.reset_chat_state(message.chat.id).await?;
                                self.process_command(message.chat.id, "/add", &[text.to_string()])
                                    .await?;
                            } else {
                                self.messenger
                                    .send_message(
                                        &self.app_postgres,
                                        &self.network_postgres,
                                        message.chat.id,
                                        Box::new(MessageType::InvalidAddressTryAgain(
                                            text.to_string(),
                                        )),
                                    )
                                    .await?;
                            }
                        }
                        TelegramChatState::EnterBugReport => {
                            log::info!("New bug report from chat id {}: {}", message.chat.id, text);
                            self.reset_chat_state(message.chat.id).await?;
                            self.network_postgres
                                .save_bug_report(message.chat.id, text)
                                .await?;
                            for admin_chat_id in &CONFIG.telegram_bot.get_admin_chat_ids() {
                                self.messenger
                                    .send_message(
                                        &self.app_postgres,
                                        &self.network_postgres,
                                        *admin_chat_id,
                                        Box::new(MessageType::BugReport(text.to_string())),
                                    )
                                    .await?;
                            }
                            self.messenger
                                .send_message(
                                    &self.app_postgres,
                                    &self.network_postgres,
                                    message.chat.id,
                                    Box::new(MessageType::ReportSaved),
                                )
                                .await?;
                        }
                        TelegramChatState::EnterFeatureRequest => {
                            log::info!(
                                "New feature request from chat id {}: {}",
                                message.chat.id,
                                text
                            );
                            self.reset_chat_state(message.chat.id).await?;
                            self.network_postgres
                                .save_feature_request(message.chat.id, text)
                                .await?;
                            for admin_chat_id in &CONFIG.telegram_bot.get_admin_chat_ids() {
                                self.messenger
                                    .send_message(
                                        &self.app_postgres,
                                        &self.network_postgres,
                                        *admin_chat_id,
                                        Box::new(MessageType::FeatureRequest(text.to_string())),
                                    )
                                    .await?;
                            }
                            self.messenger
                                .send_message(
                                    &self.app_postgres,
                                    &self.network_postgres,
                                    message.chat.id,
                                    Box::new(MessageType::ReportSaved),
                                )
                                .await?;
                        }
                        _ => {
                            if message.chat.type_field == ChatType::Private {
                                self.messenger
                                    .send_message(
                                        &self.app_postgres,
                                        &self.network_postgres,
                                        message.chat.id,
                                        Box::new(MessageType::BadRequest),
                                    )
                                    .await?;
                            }
                        }
                    }
                } else if message.chat.type_field == ChatType::Private {
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            message.chat.id,
                            Box::new(MessageType::BadRequest),
                        )
                        .await?;
                }
            }
        } else {
            self.reset_chat_state(message.chat.id).await?;
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    message.chat.id,
                    Box::new(MessageType::BadRequest),
                )
                .await?;
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl<M: Messenger + Send + Sync> Service for TelegramBot<M> {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.telegram_bot_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!("Telegram bot has started.");
        // only receive message and callback query updates
        let mut update_params = GetUpdatesParams {
            offset: None,
            limit: None,
            timeout: None,
            allowed_updates: Some(vec![
                frankenstein::types::AllowedUpdate::Message,
                frankenstein::types::AllowedUpdate::CallbackQuery,
            ]),
        };
        // update metrics
        self.update_metrics_chat_count().await?;
        self.update_metrics_validator_count().await?;
        // process Telegram updates in a loop
        loop {
            let result = self.api.get_updates(&update_params).await;
            match result {
                Ok(response) => {
                    for update in response.result {
                        update_params.offset = Some((update.update_id + 1).into());
                        match update.content {
                            // process message
                            frankenstein::updates::UpdateContent::Message(message) => {
                                self.save_or_restore_chat(message.chat.id).await?;
                                tokio::spawn(async move {
                                    if let Err(error) = self.process_message(&message).await {
                                        log::error!(
                                            "Error while processing message #{}: {:?}",
                                            message.message_id,
                                            error
                                        );
                                        let _ = self
                                            .messenger
                                            .send_message(
                                                &self.app_postgres,
                                                &self.network_postgres,
                                                message.chat.id,
                                                Box::new(MessageType::GenericError),
                                            )
                                            .await;
                                    }
                                });
                            }
                            // process callback query
                            frankenstein::updates::UpdateContent::CallbackQuery(callback_query) => {
                                if let Some(callback_data) = &callback_query.data {
                                    if let Some(MaybeInaccessibleMessage::Message(message)) =
                                        &callback_query.message
                                    {
                                        let callback_data = callback_data.clone();
                                        let message = message.clone();
                                        self.save_or_restore_chat(message.chat.id).await?;
                                        tokio::spawn(async move {
                                            let query: Query = if let Ok(query) =
                                                serde_json::from_str(&callback_data)
                                            {
                                                query
                                            } else {
                                                // log and ignore unknown query
                                                return log::error!(
                                                    "Unknown query: {callback_data}",
                                                );
                                            };
                                            if let Err(error) = tokio::try_join!(
                                                self.reset_chat_state(message.chat.id),
                                                self.process_query(
                                                    message.chat.id,
                                                    Some(message.message_id),
                                                    &query
                                                ),
                                                self.messenger.answer_callback_query(
                                                    &callback_query.id,
                                                    None
                                                ),
                                            ) {
                                                log::error!(
                                                    "Error while processing message #{}: {:?}",
                                                    message.message_id,
                                                    error
                                                );
                                                let _ = self
                                                    .messenger
                                                    .send_message(
                                                        &self.app_postgres,
                                                        &self.network_postgres,
                                                        message.chat.id,
                                                        Box::new(MessageType::GenericError),
                                                    )
                                                    .await;
                                            }
                                        });
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                Err(error) => {
                    log::error!("Error while receiving updates: {error:?}");
                }
            }
        }
    }
}

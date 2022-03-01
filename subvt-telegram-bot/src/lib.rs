//! Telegram bot. Former 1KV Telegram Bot migrated to SubVT.

use std::collections::HashSet;

use crate::messenger::{MessageType, Messenger};
use crate::query::Query;
use async_trait::async_trait;
use frankenstein::{AsyncApi, AsyncTelegramApi, ChatType, GetUpdatesParams, Message};
use lazy_static::lazy_static;
use log::{debug, error, info};
use regex::Regex;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_persistence::redis::Redis;
use subvt_service_common::Service;
use subvt_types::app::{
    NotificationPeriodType, NotificationTypeCode, User, UserNotificationChannel,
};
use subvt_types::crypto::AccountId;
use subvt_types::telegram::TelegramChatState;

mod command;
mod messenger;
mod query;

lazy_static! {
    static ref CONFIG: Config = Config::default();
    static ref CMD_REGEX: Regex = Regex::new(r"^/([a-zA-Z0-9_]+)(\s+[a-zA-Z0-9_-]+)*").unwrap();
    static ref CMD_ARG_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    static ref SPLITTER_REGEX: Regex = Regex::new(r"\s+").unwrap();
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum TelegramBotError {
    #[error("Telegram bot error: {0}")]
    Error(String),
}

pub struct TelegramBot {
    app_postgres: PostgreSQLAppStorage,
    network_postgres: PostgreSQLNetworkStorage,
    redis: Redis,
    api: AsyncApi,
    messenger: Messenger,
}

impl TelegramBot {
    pub async fn new() -> anyhow::Result<Self> {
        let app_postgres =
            PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?;
        let network_postgres =
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
        let redis = Redis::new()?;
        let api = AsyncApi::new(&CONFIG.notification_sender.telegram_token);
        let messenger = Messenger::new(&CONFIG, api.clone())?;
        Ok(TelegramBot {
            app_postgres,
            network_postgres,
            redis,
            api,
            messenger,
        })
    }
}

impl TelegramBot {
    async fn reset_chat_state(&self, telegram_chat_id: i64) -> anyhow::Result<()> {
        self.network_postgres
            .set_chat_state(telegram_chat_id, TelegramChatState::Default)
            .await?;
        Ok(())
    }

    async fn create_app_user(&self, chat_id: i64) -> anyhow::Result<u32> {
        // save app user
        let app_user_id = self.app_postgres.save_user(&User::default()).await?;
        // save notification channel
        let channel_id = self
            .app_postgres
            .save_user_notification_channel(&UserNotificationChannel {
                user_id: app_user_id,
                channel_code: "telegram".to_string(),
                target: chat_id.to_string(),
                ..Default::default()
            })
            .await?;
        let mut channel_id_set = HashSet::new();
        channel_id_set.insert(channel_id);
        // save default notification rules
        self.app_postgres
            .save_user_notification_rule(
                app_user_id,
                &NotificationTypeCode::ChainValidatorBlockAuthorship.to_string(),
                (
                    Some("Telegram block authorship notification"),
                    Some("Created by the SubVT Telegram Bot."),
                ),
                (Some(CONFIG.substrate.network_id), true),
                (&NotificationPeriodType::Immediate, 0),
                (&HashSet::new(), &channel_id_set, &vec![]),
            )
            .await?;
        Ok(app_user_id)
    }

    async fn process_message(&self, message: &Message) -> anyhow::Result<()> {
        if !self
            .network_postgres
            .chat_exists_by_id(message.chat.id)
            .await?
        {
            let app_user_id = self.create_app_user(message.chat.id).await?;
            debug!(
                "Save new chat {} with app user id {}.",
                message.chat.id, app_user_id
            );
            self.network_postgres
                .save_chat(app_user_id, message.chat.id, &TelegramChatState::Default)
                .await?;
        }
        // group chat started - send intro
        if let Some(group_chat_created) = message.group_chat_created {
            if group_chat_created {
                self.messenger
                    .send_message(message.chat.id, MessageType::Intro)
                    .await?;
                return Ok(());
            }
        }
        // text message
        if let Some(text) = message.text.clone() {
            let text = text.trim();
            if CMD_REGEX.is_match(text) {
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
                self.process_command(message.chat.id, &command, &arguments)
                    .await?;
            } else {
                let maybe_state = self
                    .network_postgres
                    .get_chat_state(message.chat.id)
                    .await?;
                if let Some(state) = maybe_state {
                    match state {
                        TelegramChatState::AddValidator => {
                            if AccountId::from_ss58_check(text).is_ok() {
                                self.reset_chat_state(message.chat.id).await?;
                                self.process_command(message.chat.id, "/add", &[text.to_string()])
                                    .await?;
                            } else {
                                self.messenger
                                    .send_message(
                                        message.chat.id,
                                        MessageType::InvalidAddressTryAgain(text.to_string()),
                                    )
                                    .await?;
                            }
                        }
                        _ => {
                            if message.chat.type_field == ChatType::Private {
                                self.messenger
                                    .send_message(message.chat.id, MessageType::BadRequest)
                                    .await?;
                            }
                        }
                    }
                } else if message.chat.type_field == ChatType::Private {
                    self.messenger
                        .send_message(message.chat.id, MessageType::BadRequest)
                        .await?;
                }
            }
        } else {
            self.messenger
                .send_message(message.chat.id, MessageType::BadRequest)
                .await?;
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl Service for TelegramBot {
    async fn run(&'static self) -> anyhow::Result<()> {
        info!("Telegram bot has started.");
        let mut update_params = GetUpdatesParams {
            offset: None,
            limit: None,
            timeout: None,
            allowed_updates: Some(vec![
                "message".to_string(),
                "callback_query".to_string(),
                // "channel_post".to_string()
            ]),
        };
        loop {
            let result = self.api.get_updates(&update_params).await;
            match result {
                Ok(response) => {
                    for update in response.result {
                        update_params.offset = Some(update.update_id + 1);
                        if let Some(message) = update.message {
                            tokio::spawn(async move {
                                if let Err(error) = self.process_message(&message).await {
                                    error!(
                                        "Error while processing message #{}: {:?}",
                                        message.message_id, error
                                    );
                                    let _ = self
                                        .messenger
                                        .send_message(message.chat.id, MessageType::GenericError)
                                        .await;
                                }
                            });
                        } else if let Some(callback_query) = update.callback_query {
                            tokio::spawn(async move {
                                if let Some(callback_data) = callback_query.data {
                                    if let Some(message) = callback_query.message {
                                        let query: Query = if let Ok(query) =
                                            serde_json::from_str(&callback_data)
                                        {
                                            query
                                        } else {
                                            // log and ignore unknown query
                                            return error!("Unknown query: {}", callback_data);
                                        };
                                        if let Err(error) = tokio::try_join!(
                                            self.messenger.delete_message(
                                                message.chat.id,
                                                message.message_id
                                            ),
                                            self.messenger
                                                .answer_callback_query(&callback_query.id, None),
                                            self.process_query(message.chat.id, &query),
                                        ) {
                                            error!(
                                                "Error while processing message #{}: {:?}",
                                                message.message_id, error
                                            );
                                            let _ = self
                                                .messenger
                                                .send_message(
                                                    message.chat.id,
                                                    MessageType::GenericError,
                                                )
                                                .await;
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
                Err(error) => {
                    error!("Error while receiving updates: {:?}", error);
                }
            }
        }
    }
}

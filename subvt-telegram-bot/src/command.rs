use crate::query::QueryType;
use crate::{MessageType, Query, TelegramBot, CONFIG};
use log::info;
use std::cmp::Ordering;
use subvt_types::app::UserValidator;
use subvt_types::crypto::AccountId;
use subvt_types::telegram::TelegramChatState;

impl TelegramBot {
    async fn process_validators_command(
        &self,
        chat_id: i64,
        query_type: QueryType,
    ) -> anyhow::Result<()> {
        let mut validators = self.network_postgres.get_chat_validators(chat_id).await?;
        if validators.is_empty() {
            self.messenger
                .send_message(chat_id, MessageType::NoValidatorsOnChat)
                .await?;
        } else if validators.len() == 1 {
            let query = Query {
                query_type,
                parameter: Some(validators.get(0).unwrap().account_id.to_ss58_check()),
            };
            self.process_query(chat_id, &query).await?;
        } else {
            info!("Send validator list for query: {}", query_type);
            validators.sort_by(|v1, v2| {
                if let Some(v1_display) = &v1.display {
                    if let Some(v2_display) = &v2.display {
                        v1_display.cmp(v2_display)
                    } else {
                        Ordering::Less
                    }
                } else if v2.display.is_some() {
                    Ordering::Greater
                } else {
                    v1.address.cmp(&v2.address)
                }
            });
            self.messenger
                .send_message(
                    chat_id,
                    MessageType::ValidatorList {
                        validators,
                        query_type,
                    },
                )
                .await?;
        }
        Ok(())
    }

    async fn process_add_command(&self, chat_id: i64, args: &[String]) -> anyhow::Result<()> {
        let validator_count = self
            .network_postgres
            .get_chat_validator_count(chat_id)
            .await?;
        if validator_count >= CONFIG.telegram_bot.max_validators_per_chat {
            self.messenger
                .send_message(chat_id, MessageType::TooManyValidatorsOnChat)
                .await?;
        }
        if args.is_empty() {
            self.network_postgres
                .set_chat_state(chat_id, TelegramChatState::AddValidator)
                .await?;
            self.messenger
                .send_message(chat_id, MessageType::AddValidator)
                .await?;
            return Ok(());
        }
        for address in args {
            match AccountId::from_ss58_check(address) {
                Ok(account_id) => {
                    if let Some(validator) = self.redis.fetch_validator_details(&account_id)? {
                        if self
                            .network_postgres
                            .chat_has_validator(chat_id, &account_id)
                            .await?
                        {
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorExistsOnChat(
                                        validator.account.get_display_or_condensed_address(None),
                                    ),
                                )
                                .await?;
                        } else {
                            let id = self
                                .network_postgres
                                .add_validator_to_chat(
                                    chat_id,
                                    &account_id,
                                    &account_id.to_ss58_check(),
                                    &validator.account.get_full_display(),
                                )
                                .await?;
                            self.update_metrics_validator_count().await?;
                            info!(
                                "Validator {} added to chat #{}. Record id: {}.",
                                account_id.to_string(),
                                chat_id,
                                id
                            );
                            let app_user_id =
                                self.network_postgres.get_chat_app_user_id(chat_id).await?;
                            // add validator to the app user for notifications
                            self.app_postgres
                                .save_user_validator(&UserValidator {
                                    id: 0,
                                    user_id: app_user_id,
                                    network_id: CONFIG.substrate.network_id,
                                    validator_account_id: account_id,
                                })
                                .await?;
                            let query = Query {
                                query_type: QueryType::ValidatorInfo,
                                parameter: Some(account_id.to_ss58_check()),
                            };
                            self.process_query(chat_id, &query).await?;
                            self.messenger
                                .send_message(chat_id, MessageType::ValidatorAdded)
                                .await?;
                        }
                    } else {
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::AddValidatorNotFound(address.clone()),
                            )
                            .await?;
                    }
                }
                Err(_) => {
                    self.messenger
                        .send_message(chat_id, MessageType::InvalidAddress(address.clone()))
                        .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn process_command(
        &self,
        chat_id: i64,
        command: &str,
        args: &[String],
    ) -> anyhow::Result<()> {
        info!(
            "Process command {} for chat {} with arguments: {:?}",
            command, chat_id, args,
        );
        match command {
            "/start" => {
                crate::metrics::command_call_counter(command).inc();
                self.messenger
                    .send_message(chat_id, MessageType::Intro)
                    .await?;
            }
            "/add" => {
                crate::metrics::command_call_counter(command).inc();
                self.process_add_command(chat_id, args).await?
            }
            "/cancel" => {
                crate::metrics::command_call_counter(command).inc();
                self.reset_chat_state(chat_id).await?;
                self.messenger
                    .send_message(chat_id, MessageType::Ok)
                    .await?;
            }
            "/remove" => {
                crate::metrics::command_call_counter(command).inc();
                if let Some(validator_address) = args.get(0) {
                    if AccountId::from_ss58_check(validator_address).is_ok() {
                        self.process_query(
                            chat_id,
                            &Query {
                                query_type: QueryType::RemoveValidator,
                                parameter: Some(validator_address.clone()),
                            },
                        )
                        .await?;
                    } else {
                        self.process_validators_command(chat_id, QueryType::RemoveValidator)
                            .await?
                    }
                } else {
                    self.process_validators_command(chat_id, QueryType::RemoveValidator)
                        .await?
                }
            }
            "/validatorinfo" => {
                crate::metrics::command_call_counter(command).inc();
                self.process_validators_command(chat_id, QueryType::ValidatorInfo)
                    .await?
            }
            "/nominations" => {
                crate::metrics::command_call_counter(command).inc();
                self.process_validators_command(chat_id, QueryType::NominationSummary)
                    .await?
            }
            "/broadcasttest" => {
                crate::metrics::command_call_counter(command).inc();
                if CONFIG.telegram_bot.admin_chat_id == chat_id {
                    self.messenger
                        .send_message(chat_id, MessageType::Broadcast)
                        .await?;
                } else {
                    self.messenger
                        .send_message(chat_id, MessageType::UnknownCommand(command.to_string()))
                        .await?;
                }
            }
            "/broadcast" => {
                crate::metrics::command_call_counter(command).inc();
                if CONFIG.telegram_bot.admin_chat_id == chat_id {
                    self.messenger
                        .send_message(chat_id, MessageType::BroadcastConfirm)
                        .await?;
                } else {
                    self.messenger
                        .send_message(chat_id, MessageType::UnknownCommand(command.to_string()))
                        .await?;
                }
            }
            _ => {
                crate::metrics::command_call_counter("invalid").inc();
                self.messenger
                    .send_message(chat_id, MessageType::UnknownCommand(command.to_string()))
                    .await?;
            }
        }
        Ok(())
    }
}

use crate::query::QueryType;
use crate::{MessageType, Query, TelegramBot};
use log::info;
use std::cmp::Ordering;
use subvt_types::crypto::AccountId;
use subvt_types::telegram::TelegramChatState;

impl TelegramBot {
    async fn process_validators_command(
        &self,
        chat_id: i64,
        query_type: QueryType,
    ) -> anyhow::Result<()> {
        let validator_account_ids = self
            .postgres
            .get_chat_validator_account_ids(chat_id)
            .await?;
        if validator_account_ids.is_empty() {
            self.messenger
                .send_message(chat_id, MessageType::NoValidatorsOnChat)
                .await?;
        } else if validator_account_ids.len() == 1 {
            let query = Query {
                query_type,
                parameter: Some(validator_account_ids.get(0).unwrap().to_ss58_check()),
            };
            self.process_query(chat_id, &query).await?;
        } else {
            // multiple validators
            let mut validators = Vec::new();
            for account_id in &validator_account_ids {
                validators.push(self.redis.fetch_validator_details(account_id)?);
            }
            validators.sort_by(|v1, v2| {
                let maybe_v1_display = v1.get_display();
                let maybe_v2_display = v2.get_display();
                if let Some(v1_display) = maybe_v1_display {
                    if let Some(v2_display) = maybe_v2_display {
                        v1_display.cmp(&v2_display)
                    } else {
                        Ordering::Less
                    }
                } else if maybe_v2_display.is_some() {
                    Ordering::Greater
                } else {
                    v1.account.address.cmp(&v2.account.address)
                }
            });
            self.messenger
                .send_message(chat_id, MessageType::ValidatorList(validators, query_type))
                .await?;
        }
        Ok(())
    }

    async fn process_add_command(&self, chat_id: i64, args: &[String]) -> anyhow::Result<()> {
        if args.is_empty() {
            self.postgres
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
                    if self.redis.validator_exists_by_account_id(&account_id)? {
                        if self
                            .postgres
                            .chat_has_validator(chat_id, &account_id)
                            .await?
                        {
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorExistsOnChat(address.clone()),
                                )
                                .await?;
                        } else {
                            let id = self
                                .postgres
                                .add_validator_to_chat(chat_id, &account_id)
                                .await?;
                            info!(
                                "Validator {} added to chat #{}. Record id: {}.",
                                account_id.to_string(),
                                chat_id,
                                id
                            );
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
                            .send_message(chat_id, MessageType::ValidatorNotFound(address.clone()))
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
        match command {
            "/start" => {
                self.messenger
                    .send_message(chat_id, MessageType::Intro)
                    .await?;
            }
            "/add" => self.process_add_command(chat_id, args).await?,
            "/validator_info" => {
                self.process_validators_command(chat_id, QueryType::ValidatorInfo)
                    .await?
            }
            "/nominations" => {
                self.process_validators_command(chat_id, QueryType::Nominations)
                    .await?
            }
            _ => {
                self.messenger
                    .send_message(chat_id, MessageType::UnknownCommand(command.to_string()))
                    .await?;
            }
        }
        Ok(())
    }
}

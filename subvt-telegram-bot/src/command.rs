use crate::{MessageType, TelegramBot};
use log::info;
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::telegram::TelegramChatState;

impl TelegramBot {
    async fn process_add(
        &self,
        config: &Config,
        chat_id: i64,
        args: &[String],
    ) -> anyhow::Result<()> {
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
                            let validator_details =
                                self.redis.fetch_validator_details(&account_id)?;
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorAdded {
                                        network: config.substrate.chain.clone(),
                                        address: address.clone(),
                                        validator_details: Box::new(validator_details),
                                    },
                                )
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
        // check address (only 1st param) :: check redis, check postgres (era validator)
        // save if exists & send message
        Ok(())
    }

    pub async fn process_command(
        &self,
        config: &Config,
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
            "/add" => self.process_add(config, chat_id, args).await?,
            _ => {
                self.messenger
                    .send_message(chat_id, MessageType::UnknownCommand(command.to_string()))
                    .await?;
            }
        }
        Ok(())
    }
}

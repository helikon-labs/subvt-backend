use crate::query::QueryType;
use crate::{MessageType, TelegramBot};
use log::info;
use subvt_types::crypto::AccountId;
use subvt_types::telegram::TelegramChatState;

impl TelegramBot {
    async fn process_validator_info_command(&self, chat_id: i64) -> anyhow::Result<()> {
        let validator_account_ids = self
            .postgres
            .get_chat_validator_account_ids(chat_id)
            .await?;
        if validator_account_ids.is_empty() {
            println!("no validators");
            return Ok(());
        }
        if validator_account_ids.len() == 1 {
            let validator_details = self
                .redis
                .fetch_validator_details(validator_account_ids.get(0).unwrap())?;
            self.messenger
                .send_message(
                    chat_id,
                    MessageType::ValidatorInfo(Box::new(validator_details)),
                )
                .await?;
        } else {
            // multiple validators
            let mut validators = Vec::new();
            for account_id in &validator_account_ids {
                validators.push(self.redis.fetch_validator_details(account_id)?);
            }
            self.messenger
                .send_validator_list(chat_id, QueryType::ValidatorInfo, &validators)
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
                            let validator_details =
                                self.redis.fetch_validator_details(&account_id)?;
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorInfo(Box::new(validator_details)),
                                )
                                .await?;
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
            "/validator_info" => self.process_validator_info_command(chat_id).await?,
            _ => {
                self.messenger
                    .send_message(chat_id, MessageType::UnknownCommand(command.to_string()))
                    .await?;
            }
        }
        Ok(())
    }
}

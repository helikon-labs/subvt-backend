use super::{MessageType, TelegramBot};
use crate::{
    query::{Query, QueryType},
    CONFIG,
};
use subvt_types::app::UserValidator;
use subvt_types::crypto::AccountId;
use subvt_types::telegram::TelegramChatState;

impl TelegramBot {
    pub(crate) async fn process_add_validator_command(
        &self,
        chat_id: i64,
        args: &[String],
    ) -> anyhow::Result<()> {
        let validator_count = self
            .network_postgres
            .get_chat_validator_count(chat_id)
            .await?;
        if validator_count >= CONFIG.telegram_bot.max_validators_per_chat {
            self.messenger
                .send_message(
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::TooManyValidatorsOnChat),
                )
                .await?;
        }
        if args.is_empty() {
            self.network_postgres
                .set_chat_state(chat_id, TelegramChatState::AddValidator)
                .await?;
            self.messenger
                .send_message(
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::AddValidator),
                )
                .await?;
            return Ok(());
        }
        for address in args {
            match AccountId::from_ss58_check(address) {
                Ok(account_id) => {
                    if let Some(validator) = self.redis.fetch_validator_details(&account_id).await?
                    {
                        if self
                            .network_postgres
                            .chat_has_validator(chat_id, &account_id)
                            .await?
                        {
                            self.messenger
                                .send_message(
                                    &self.network_postgres,
                                    chat_id,
                                    Box::new(MessageType::ValidatorExistsOnChat(
                                        validator.account.get_display_or_condensed_address(None),
                                    )),
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
                            log::info!(
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
                                parameter: Some(id.to_string()),
                            };
                            self.process_query(chat_id, None, &query).await?;
                            self.messenger
                                .send_message(
                                    &self.network_postgres,
                                    chat_id,
                                    Box::new(MessageType::ValidatorAdded),
                                )
                                .await?;
                        }
                    } else {
                        self.messenger
                            .send_message(
                                &self.network_postgres,
                                chat_id,
                                Box::new(MessageType::AddValidatorNotFound(address.clone())),
                            )
                            .await?;
                    }
                }
                Err(_) => {
                    self.messenger
                        .send_message(
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::InvalidAddress(address.clone())),
                        )
                        .await?;
                }
            }
        }
        Ok(())
    }
}

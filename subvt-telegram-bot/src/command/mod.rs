//! Telegram bot command processing happens here. All the commands are `match`ed and processed in
//! the `process_command` function below and the corresponding command modules.
use crate::{query::QueryType, MessageType, Messenger, TelegramBot};
use async_recursion::async_recursion;

mod add_validator;
mod broadcast;
mod broadcast_test;
mod network_status;
mod opengov;
mod payouts;
mod remove_all_validators;
mod remove_validator;
mod rewards;
mod settings;
mod summary;
mod validators;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    #[async_recursion]
    pub async fn process_command(
        &self,
        chat_id: i64,
        command: &str,
        args: &[String],
    ) -> anyhow::Result<()> {
        log::info!(
            "Process command {command} for chat {chat_id} with arguments: {args:?}",
        );
        match command {
            "/about" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::About),
                    )
                    .await?;
            }
            "/help" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::Help),
                    )
                    .await?;
            }
            "/start" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::Intro),
                    )
                    .await?;
            }
            "/add" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_add_validator_command(chat_id, args).await?;
            }
            "/cancel" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.reset_chat_state(chat_id).await?;
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::Ok),
                    )
                    .await?;
            }
            "/networkstatus" | "/network" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_network_status_command(chat_id).await?;
            }
            "/validatorinfo" | "/vi" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_validators_command(chat_id, QueryType::ValidatorInfo)
                    .await?;
            }
            "/nominations" | "/n" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_validators_command(chat_id, QueryType::NominationSummary)
                    .await?;
            }
            "/nominationdetails" | "/nd" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_validators_command(chat_id, QueryType::NominationDetails)
                    .await?;
            }
            "/payouts" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_payouts_command(chat_id, args).await?;
            }
            "/referenda" | "/opengov" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_opengov_command(chat_id).await?;
            }
            "/remove" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_remove_validator_command(chat_id, args).await?;
            }
            "/removeall" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_remove_all_validators_command(chat_id).await?;
            }
            "/contact" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::SelectContactType),
                    )
                    .await?;
            }
            "/rewards" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_rewards_command(chat_id, args).await?;
            }
            "/settings" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_settings_command(chat_id).await?;
            }
            "/broadcasttest" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_broadcast_test_command(chat_id, command)
                    .await?;
            }
            "/broadcast" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_broadcast_command(chat_id, command).await?;
            }
            "/nfts" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_validators_command(chat_id, QueryType::NFTs(0, true))
                    .await?;
            }
            "/summary" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_summary_command(chat_id).await?;
            }
            _ => {
                crate::metrics::command_call_counter("invalid").inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, "invalid")
                    .await?;
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::UnknownCommand(command.to_string())),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

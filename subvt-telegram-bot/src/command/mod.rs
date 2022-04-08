use crate::{query::QueryType, MessageType, TelegramBot};

mod add_validator;
mod broadcast;
mod broadcast_test;
mod network_status;
mod remove_validator;
mod settings;
mod validators;

impl TelegramBot {
    pub async fn process_command(
        &self,
        chat_id: i64,
        command: &str,
        args: &[String],
    ) -> anyhow::Result<()> {
        log::info!(
            "Process command {} for chat {} with arguments: {:?}",
            command,
            chat_id,
            args,
        );
        match command {
            "/start" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.messenger
                    .send_message(
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
                    .send_message(&self.network_postgres, chat_id, Box::new(MessageType::Ok))
                    .await?;
            }
            "/remove" => {
                crate::metrics::command_call_counter(command).inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, command)
                    .await?;
                self.process_remove_validator_command(chat_id, args).await?;
            }
            "/networkstatus" | "/network" | "/netstat" | "/ns" => {
                crate::metrics::command_call_counter(command).inc();
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
            "/settings" | "/s" => {
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
            _ => {
                crate::metrics::command_call_counter("invalid").inc();
                self.network_postgres
                    .save_chat_command_log(chat_id, "invalid")
                    .await?;
                self.messenger
                    .send_message(
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

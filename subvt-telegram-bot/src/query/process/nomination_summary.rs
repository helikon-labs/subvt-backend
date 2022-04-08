use crate::query::Query;
use crate::{messenger::message::MessageType, TelegramBot};

impl TelegramBot {
    pub(crate) async fn process_nomination_summary_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        if let Some(id_str) = &query.parameter {
            if let Some(validator) = self
                .network_postgres
                .get_chat_validator_by_id(chat_id, id_str.parse()?)
                .await?
            {
                if let Some(validator_details) = self
                    .redis
                    .fetch_validator_details(&validator.account_id)
                    .await?
                {
                    log::info!(
                        "Validator selected for nomination summary in chat {}.",
                        chat_id
                    );
                    self.network_postgres
                        .update_chat_validator_display(
                            &validator.account_id,
                            &validator_details.account.get_full_display(),
                        )
                        .await?;
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::NominationSummary {
                                validator_details,
                                chat_validator_id: id_str.parse()?,
                            }),
                        )
                        .await?;
                } else {
                    log::warn!(
                        "Validator not found! Selected for nomination summary in chat {}.",
                        chat_id
                    );
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::ValidatorNotFound {
                                maybe_address: None,
                            }),
                        )
                        .await?;
                }
            } else {
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::ValidatorNotFound {
                            maybe_address: None,
                        }),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

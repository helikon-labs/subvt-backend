use crate::query::Query;
use crate::{messenger::message::MessageType, TelegramBot};

impl TelegramBot {
    pub(crate) async fn process_validator_info_query(
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
                let maybe_validator_details = self
                    .redis
                    .fetch_validator_details(&validator.account_id)
                    .await?;
                if let Some(validator_details) = &maybe_validator_details {
                    self.network_postgres
                        .update_chat_validator_display(
                            &validator.account_id,
                            &validator_details.account.get_full_display(),
                        )
                        .await?;
                }
                self.messenger
                    .send_message(
                        chat_id,
                        Box::new(MessageType::ValidatorInfo {
                            address: validator.address.clone(),
                            maybe_validator_details: Box::new(maybe_validator_details),
                            maybe_onekv_candidate_summary: Box::new(
                                self.network_postgres
                                    .get_onekv_candidate_summary_by_account_id(
                                        &validator.account_id,
                                    )
                                    .await?,
                            ),
                        }),
                    )
                    .await?;
            } else {
                self.messenger
                    .send_message(
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

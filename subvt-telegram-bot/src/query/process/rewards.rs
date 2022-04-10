use crate::query::Query;
use crate::{messenger::message::MessageType, TelegramBot};

impl TelegramBot {
    pub(crate) async fn process_rewards_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        if let Some(id_str) = &query.parameter {
            log::info!("Validator selected for rewards in chat {}.", chat_id);
            if let Some(validator) = self
                .network_postgres
                .get_chat_validator_by_id(chat_id, id_str.parse()?)
                .await?
            {
                let era_rewards = self
                    .network_postgres
                    .get_validator_rewards(&validator.account_id)
                    .await?;
                if era_rewards.is_empty() {
                    println!("no rewards for {}", validator.address);
                } else {
                    println!("send rewards report for {}", validator.address);
                    /*
                    prepare data structure
                    prepare rewards report
                    generate plot image, return the path/file
                    send the image file
                    */
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

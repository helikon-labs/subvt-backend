use crate::query::Query;
use crate::{messenger::message::MessageType, Messenger, TelegramBot, CONFIG};

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub(crate) async fn process_remove_validator_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        if let Some(id_str) = &query.parameter {
            log::info!("Validator selected for removal in chat {chat_id}.");
            if let Some(validator) = self
                .network_postgres
                .get_chat_validator_by_id(chat_id, id_str.parse()?)
                .await?
            {
                if self
                    .network_postgres
                    .remove_validator_from_chat(chat_id, &validator.account_id)
                    .await?
                {
                    self.update_metrics_validator_count().await?;
                    let app_user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
                    // remove from app, so it doesn't receive notifications
                    let _ = self
                        .app_postgres
                        .delete_user_validator_by_account_id(
                            app_user_id,
                            CONFIG.substrate.network_id,
                            &validator.account_id,
                        )
                        .await?;
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::ValidatorRemoved(validator)),
                        )
                        .await?;
                } else {
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::GenericError),
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

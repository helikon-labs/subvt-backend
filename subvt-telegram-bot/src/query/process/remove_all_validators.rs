use crate::{messenger::message::MessageType, Messenger, TelegramBot, CONFIG};

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub(crate) async fn process_remove_all_validators_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        let chat_validators = self.network_postgres.get_chat_validators(chat_id).await?;
        let app_user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
        for chat_validator in chat_validators {
            if self
                .network_postgres
                .remove_validator_from_chat(chat_id, &chat_validator.account_id)
                .await?
            {
                // remove from app, so it doesn't receive notifications
                self.app_postgres
                    .delete_user_validator_by_account_id(
                        app_user_id,
                        CONFIG.substrate.network_id,
                        &chat_validator.account_id,
                    )
                    .await?;
            }
        }
        // send success message
        self.messenger
            .send_message(
                &self.app_postgres,
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::AllValidatorsRemoved),
            )
            .await?;
        Ok(())
    }
}

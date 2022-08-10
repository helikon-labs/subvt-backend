//! `/removeall` command processor.
use crate::query::QueryType;
use crate::{MessageType, Messenger, Query, TelegramBot};

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    //! Removes all validators from the chat. Sends a prompt if there are more than 1 validator.
    pub(crate) async fn process_remove_all_validators_command(
        &self,
        chat_id: i64,
    ) -> anyhow::Result<()> {
        let validators = self.network_postgres.get_chat_validators(chat_id).await?;
        if validators.is_empty() {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::NoValidatorsOnChat),
                )
                .await?;
        } else if validators.len() == 1 {
            self.process_query(
                chat_id,
                None,
                &Query {
                    query_type: QueryType::RemoveValidator,
                    parameter: Some(validators[0].id.to_string()),
                },
            )
            .await?;
        } else {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::RemoveAllValidatorsConfirm),
                )
                .await?;
        }
        Ok(())
    }
}

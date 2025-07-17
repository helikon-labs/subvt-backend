//! A utility command to send the user the validator selection for the selected command,
//! such as `/rewards` or `/validatorinfo`.
use crate::query::QueryType;
use crate::{MessageType, Messenger, Query, TelegramBot};
use std::cmp::Ordering;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    //! Send the user a list of all validators on the chat, so they can select one for the selected
    //! operation such as `/rewards`, or `/validatorinfo`.
    pub(crate) async fn process_validators_command(
        &self,
        chat_id: i64,
        query_type: QueryType,
    ) -> anyhow::Result<()> {
        let mut validators = self.network_postgres.get_chat_validators(chat_id).await?;
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
            let query = Query {
                query_type,
                parameter: Some(validators.first().unwrap().id.to_string()),
            };
            self.process_query(chat_id, None, &query).await?;
        } else {
            log::info!("Send validator list for query: {query_type:?}");
            validators.sort_by(|v1, v2| {
                if let Some(v1_display) = &v1.display {
                    if let Some(v2_display) = &v2.display {
                        v1_display.cmp(v2_display)
                    } else {
                        Ordering::Less
                    }
                } else if v2.display.is_some() {
                    Ordering::Greater
                } else {
                    v1.address.cmp(&v2.address)
                }
            });
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::ValidatorList {
                        validators,
                        query_type,
                    }),
                )
                .await?;
        }
        Ok(())
    }
}

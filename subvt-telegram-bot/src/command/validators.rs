use crate::query::QueryType;
use crate::{MessageType, Query, TelegramBot};
use std::cmp::Ordering;

impl TelegramBot {
    pub(crate) async fn process_validators_command(
        &self,
        chat_id: i64,
        query_type: QueryType,
    ) -> anyhow::Result<()> {
        let mut validators = self.network_postgres.get_chat_validators(chat_id).await?;
        if validators.is_empty() {
            self.messenger
                .send_message(
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::NoValidatorsOnChat),
                )
                .await?;
        } else if validators.len() == 1 {
            let query = Query {
                query_type,
                parameter: Some(validators.get(0).unwrap().id.to_string()),
            };
            self.process_query(chat_id, None, &query).await?;
        } else {
            log::info!("Send validator list for query: {:?}", query_type);
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

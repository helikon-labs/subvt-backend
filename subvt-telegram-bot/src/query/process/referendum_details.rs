use crate::query::Query;
use crate::{MessageType, Messenger, TelegramBot};
use subvt_governance::polkassembly;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub(crate) async fn process_referendum_details_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        if let Some(id_str) = &query.parameter {
            let referendum_id: u32 = id_str.parse()?;
            if let Some(post) = polkassembly::fetch_referendum_details(referendum_id).await? {
                let chat_validators = self.network_postgres.get_chat_validators(chat_id).await?;
                let validator_votes = self
                    .network_postgres
                    .get_chat_validator_votes_for_referendum(chat_id, referendum_id)
                    .await?;
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::ReferendumDetails {
                            post,
                            chat_validators,
                            validator_votes,
                        }),
                    )
                    .await?;
            } else {
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::ReferendumNotFound(referendum_id)),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

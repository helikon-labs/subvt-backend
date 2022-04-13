use crate::query::Query;
use crate::{MessageType, TelegramBot};
use subvt_governance::polkassembly;

impl TelegramBot {
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
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::ReferendumDetails(post)),
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

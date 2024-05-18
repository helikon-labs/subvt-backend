use crate::query::Query;
use crate::{MessageType, Messenger, TelegramBot, CONFIG};
use subvt_governance::polkassembly;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::democracy::ReferendumVote;
use subvt_types::telegram::TelegramChatValidator;

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
        if let Some(params_str) = &query.parameter {
            let (track_id, referendum_id_str) = serde_json::from_str::<(u16, String)>(params_str)?;
            let referendum_id: u32 = referendum_id_str.parse()?;
            let post = polkassembly::fetch_referendum_details(referendum_id).await?;
            let chat_validators = self.network_postgres.get_chat_validators(chat_id).await?;
            let mut chat_validator_votes: Vec<(TelegramChatValidator, Option<ReferendumVote>)> =
                vec![];
            let substrate_client = SubstrateClient::new(
                CONFIG.substrate.rpc_url.as_str(),
                CONFIG.substrate.network_id,
                CONFIG.substrate.connection_timeout_seconds,
                CONFIG.substrate.request_timeout_seconds,
            )
            .await?;
            for chat_validator in &chat_validators {
                let vote = substrate_client
                    .get_account_referendum_conviction_vote(
                        &chat_validator.account_id,
                        track_id,
                        referendum_id,
                        None,
                    )
                    .await?;
                chat_validator_votes.push((chat_validator.clone(), vote));
            }
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::ReferendumDetails {
                        post,
                        chat_validator_votes,
                    }),
                )
                .await?;
        }
        Ok(())
    }
}

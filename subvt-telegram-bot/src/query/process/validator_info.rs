use crate::query::Query;
use crate::{messenger::message::MessageType, Messenger, TelegramBot, CONFIG};
use subvt_substrate_client::SubstrateClient;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
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
                    .fetch_validator_details(
                        self.redis.get_finalized_block_summary().await?.number,
                        &validator.account_id,
                    )
                    .await?;
                if let Some(validator_details) = &maybe_validator_details {
                    self.network_postgres
                        .update_chat_validator_display(
                            &validator.account_id,
                            &validator_details.account.get_full_display(),
                        )
                        .await?;
                }
                let referenda = self.network_postgres.get_open_referenda(None).await?;
                let substrate_client = SubstrateClient::new(
                    CONFIG.substrate.rpc_url.as_str(),
                    CONFIG.substrate.network_id,
                    CONFIG.substrate.connection_timeout_seconds,
                    CONFIG.substrate.request_timeout_seconds,
                )
                .await?;
                let mut missing_referendum_votes: Vec<u32> = vec![];
                for referendum in &referenda {
                    if substrate_client
                        .get_account_referendum_conviction_vote(
                            &validator.account_id,
                            referendum.track_no,
                            referendum.post_id,
                            None,
                        )
                        .await?
                        .is_none()
                    {
                        missing_referendum_votes.push(referendum.post_id);
                    }
                }
                missing_referendum_votes.sort_unstable();
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
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
                            missing_referendum_votes,
                        }),
                    )
                    .await?;
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

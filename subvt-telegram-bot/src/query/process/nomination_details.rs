use crate::query::Query;
use crate::{messenger::message::MessageType, Messenger, TelegramBot};

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub(crate) async fn process_nomination_details_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
        is_full: bool,
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
                if let Some(validator_details) = self
                    .redis
                    .fetch_validator_details(
                        self.redis.get_finalized_block_summary().await?.number,
                        &validator.account_id,
                    )
                    .await?
                {
                    log::info!("Validator selected for nomination details in chat {chat_id}.",);
                    let onekv_nominator_account_ids = self
                        .network_postgres
                        .get_onekv_nominator_stash_account_ids()
                        .await?;
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::NominationDetails {
                                chat_validator_id: validator.id,
                                validator_details,
                                onekv_nominator_account_ids,
                                is_full,
                            }),
                        )
                        .await?;
                } else {
                    log::warn!(
                        "Validator not found! Selected for nomination details in chat {chat_id}.",
                    );
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

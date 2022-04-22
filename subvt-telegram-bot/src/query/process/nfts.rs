use crate::query::Query;
use crate::{MessageType, TelegramBot};

const PAGE_SIZE: usize = 7;

impl TelegramBot {
    pub(crate) async fn process_nfts_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
        page_index: usize,
        is_first_load: bool,
    ) -> anyhow::Result<()> {
        if is_first_load {
            if let Some(message_id) = original_message_id {
                self.messenger.delete_message(chat_id, message_id).await?;
            }
        }
        if let Some(id_str) = &query.parameter {
            log::info!("Validator selected for NFTs in chat {}.", chat_id);
            let validator_id: u64 = id_str.parse()?;
            if let Some(validator) = self
                .network_postgres
                .get_chat_validator_by_id(chat_id, validator_id)
                .await?
            {
                if is_first_load {
                    let response = self
                        .messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::Loading),
                        )
                        .await?;
                    let collection = subvt_nft::get_account_nfts(&validator.address).await?;
                    self.network_postgres
                        .save_nft_collection(&validator.account_id, &collection)
                        .await?;
                    self.messenger
                        .delete_message(chat_id, response.result.message_id)
                        .await?;
                }
                // query with limit
                let collection_page = self
                    .network_postgres
                    .get_account_nfts(&validator.account_id, page_index, PAGE_SIZE)
                    .await?;
                let total_count = self
                    .network_postgres
                    .get_account_nft_count(&validator.account_id)
                    .await?;
                if total_count == 0 {
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::NoNFTsForValidator),
                        )
                        .await?;
                    return Ok(());
                }
                let has_prev = page_index != 0;
                let has_next = (page_index + 1) * PAGE_SIZE < total_count;
                if is_first_load {
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::NFTs {
                                validator_id,
                                total_count,
                                collection_page,
                                page_index,
                                has_prev,
                                has_next,
                            }),
                        )
                        .await?;
                } else if let Some(message_id) = original_message_id {
                    self.messenger
                        .update_nfts_message(
                            chat_id,
                            message_id,
                            validator_id,
                            total_count,
                            collection_page,
                            page_index,
                            has_prev,
                            has_next,
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

use crate::query::{Query, QueryType};
use crate::{Messenger, TelegramBot};

mod broadcast;
mod nfts;
mod nomination_details;
mod nomination_summary;
mod payouts;
mod referendum_details;
mod remove_all_validators;
mod remove_validator;
mod report_bug;
mod report_feature_request;
mod rewards;
mod settings;
mod settings_navigate;
mod validator_info;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub async fn process_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        log::info!("Process query: {:?}", query);
        crate::metrics::query_call_counter(&query.query_type).inc();
        self.network_postgres
            .save_chat_query_log(chat_id, &format!("{:?}", query))
            .await?;
        match &query.query_type {
            QueryType::NoOp => (),
            QueryType::Cancel | QueryType::Close => {
                if let Some(message_id) = original_message_id {
                    self.messenger.delete_message(chat_id, message_id).await?;
                }
            }
            QueryType::ConfirmBroadcast => {
                self.process_confirm_broadcast_query(chat_id, original_message_id)
                    .await?;
            }
            QueryType::NFTs(page_index, is_first_load) => {
                self.process_nfts_query(
                    chat_id,
                    original_message_id,
                    query,
                    *page_index,
                    *is_first_load,
                )
                .await?;
            }
            QueryType::NominationDetails => {
                self.process_nomination_details_query(chat_id, original_message_id, query)
                    .await?;
            }
            QueryType::NominationSummary => {
                self.process_nomination_summary_query(chat_id, original_message_id, query)
                    .await?;
            }
            QueryType::Payouts => {
                self.process_payouts_query(chat_id, original_message_id, query)
                    .await?;
            }
            QueryType::ReferendumDetails => {
                self.process_referendum_details_query(chat_id, original_message_id, query)
                    .await?;
            }
            QueryType::RemoveAllValidators => {
                self.process_remove_all_validators_query(chat_id, original_message_id)
                    .await?;
            }
            QueryType::RemoveValidator => {
                self.process_remove_validator_query(chat_id, original_message_id, query)
                    .await?;
            }
            QueryType::ReportBug => {
                self.process_report_bug_query(chat_id, original_message_id)
                    .await?;
            }
            QueryType::ReportFeatureRequest => {
                self.process_report_feature_request_query(chat_id, original_message_id)
                    .await?;
            }
            QueryType::Rewards => {
                self.process_rewards_query(chat_id, original_message_id, query)
                    .await?;
            }
            QueryType::SettingsEdit(edit_query_type) => {
                self.process_settings_edit_query(chat_id, query, edit_query_type)
                    .await?;
            }
            QueryType::SettingsNavigate(sub_section) => {
                self.process_settings_navigate_query(chat_id, *sub_section)
                    .await?;
            }
            QueryType::ValidatorInfo => {
                self.process_validator_info_query(chat_id, original_message_id, query)
                    .await?;
            }
        }
        Ok(())
    }
}

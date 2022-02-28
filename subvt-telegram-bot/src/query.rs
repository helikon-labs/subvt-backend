use crate::{MessageType, TelegramBot};
use log::{error, info};
use serde::{Deserialize, Serialize};
use subvt_types::crypto::AccountId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum QueryType {
    #[serde(rename = "VI")]
    ValidatorInfo,
    #[serde(rename = "NS")]
    NominationSummary,
    #[serde(rename = "ND")]
    NominationDetails,
    #[serde(rename = "RV")]
    RemoveValidator,
    #[serde(rename = "CB")]
    ConfirmBroadcast,
    #[serde(rename = "C")]
    Cancel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
    #[serde(rename = "q")]
    pub query_type: QueryType,
    #[serde(rename = "p")]
    pub parameter: Option<String>,
}

impl Query {
    pub fn get_cancel_query() -> Query {
        Query {
            query_type: QueryType::Cancel,
            parameter: None,
        }
    }
}

impl TelegramBot {
    pub async fn process_query(&self, chat_id: i64, query: &Query) -> anyhow::Result<()> {
        match query.query_type {
            QueryType::ConfirmBroadcast => {
                info!("Broadcast confirmed, sending.");
                for chat_id in self.postgres.get_chat_ids().await? {
                    match self
                        .messenger
                        .send_message(chat_id, MessageType::Broadcast)
                        .await
                    {
                        Ok(_) => info!("Broadcast sent to chat {}.", chat_id),
                        Err(error) => error!(
                            "Error while sending broadcast to chat {}: {:?}",
                            chat_id, error
                        ),
                    }
                }
            }
            QueryType::ValidatorInfo => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        let validator_details = self.redis.fetch_validator_details(&account_id)?;
                        info!("Validator selected for validator info in chat {}.", chat_id);
                        let onekv_summary =
                            if let Some(id) = validator_details.onekv_candidate_record_id {
                                self.postgres.get_onekv_candidate_summary_by_id(id).await?
                            } else {
                                None
                            };
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::ValidatorInfo(
                                    Box::new(validator_details),
                                    Box::new(onekv_summary),
                                ),
                            )
                            .await?;
                    }
                }
            }
            QueryType::NominationSummary => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        info!(
                            "Validator selected for nomination summary in chat {}.",
                            chat_id
                        );
                        let validator_details = self.redis.fetch_validator_details(&account_id)?;
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::NominationSummary(validator_details),
                            )
                            .await?;
                    }
                }
            }
            QueryType::NominationDetails => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        info!(
                            "Validator selected for nomination details in chat {}.",
                            chat_id
                        );
                        let validator_details = self.redis.fetch_validator_details(&account_id)?;
                        let onekv_nominator_account_ids =
                            self.postgres.get_onekv_nominator_account_ids().await?;
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::NominationDetails {
                                    validator_details,
                                    onekv_nominator_account_ids,
                                },
                            )
                            .await?;
                    }
                }
            }
            QueryType::RemoveValidator => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        info!("Validator selected for removal in chat {}.", chat_id);
                        if self
                            .postgres
                            .chat_has_validator(chat_id, &account_id)
                            .await?
                        {
                            if self
                                .postgres
                                .remove_validator_from_chat(chat_id, &account_id)
                                .await?
                            {
                                let validator_details =
                                    self.redis.fetch_validator_details(&account_id)?;
                                self.messenger
                                    .send_message(
                                        chat_id,
                                        MessageType::ValidatorRemoved(validator_details),
                                    )
                                    .await?;
                            } else {
                                self.messenger
                                    .send_message(chat_id, MessageType::GenericError)
                                    .await?;
                            }
                        } else {
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::RemoveValidatorNotFound(validator_address.clone()),
                                )
                                .await?;
                        }
                    }
                }
            }
            QueryType::Cancel => (),
        }
        Ok(())
    }
}

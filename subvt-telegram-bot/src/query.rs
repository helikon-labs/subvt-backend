use crate::{MessageType, TelegramBot};
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
    #[serde(rename = "q")]
    pub query_type: QueryType,
    #[serde(rename = "p")]
    pub parameter: Option<String>,
}

impl TelegramBot {
    pub async fn process_query(&self, chat_id: i64, query: &Query) -> anyhow::Result<()> {
        match query.query_type {
            QueryType::ValidatorInfo => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        let validator_details = self.redis.fetch_validator_details(&account_id)?;
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
                        let validator_details = self.redis.fetch_validator_details(&account_id)?;
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::NominationDetails(validator_details),
                            )
                            .await?;
                    }
                }
            }
            QueryType::RemoveValidator => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        // call postgres
                        // validator exists :: delete & return
                        // not exists :: message
                    }
                }
            }
        }
        Ok(())
    }
}

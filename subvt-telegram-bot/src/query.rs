use crate::{MessageType, TelegramBot, CONFIG};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
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

impl Display for QueryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Self::ValidatorInfo => "ValidatorInfo",
            Self::NominationSummary => "NominationSummary",
            Self::NominationDetails => "NominationDetails",
            Self::RemoveValidator => "RemoveValidator",
            Self::ConfirmBroadcast => "ConfirmBroadcast",
            Self::Cancel => "Cancel",
        };
        write!(f, "{}", display)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Query {
    #[serde(rename = "q")]
    pub query_type: QueryType,
    #[serde(rename = "p")]
    pub parameter: Option<String>,
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?})", self.query_type, self.parameter)
    }
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
        log::info!("Process query: {}", query);
        crate::metrics::query_call_counter(&query.query_type).inc();
        match query.query_type {
            QueryType::ConfirmBroadcast => {
                log::info!("Broadcast confirmed, sending.");
                for chat_id in self.network_postgres.get_chat_ids().await? {
                    match self
                        .messenger
                        .send_message(chat_id, MessageType::Broadcast)
                        .await
                    {
                        Ok(_) => log::info!("Broadcast sent to chat {}.", chat_id),
                        Err(error) => log::error!(
                            "Error while sending broadcast to chat {}: {:?}",
                            chat_id,
                            error
                        ),
                    }
                }
            }
            QueryType::ValidatorInfo => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        if let Some(validator_details) =
                            self.redis.fetch_validator_details(&account_id)?
                        {
                            log::info!(
                                "Validator selected for validator info in chat {}.",
                                chat_id
                            );
                            let onekv_summary =
                                if let Some(id) = validator_details.onekv_candidate_record_id {
                                    self.network_postgres
                                        .get_onekv_candidate_summary_by_id(id)
                                        .await?
                                } else {
                                    None
                                };
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorInfo(
                                        Box::new(validator_details.clone()),
                                        Box::new(onekv_summary),
                                    ),
                                )
                                .await?;
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::NominationSummary(validator_details),
                                )
                                .await?;
                        } else {
                            log::warn!(
                                "Validator not found! Selected for validator info in chat {}.",
                                chat_id
                            );
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorNotFound(validator_address.clone()),
                                )
                                .await?;
                        }
                    }
                }
            }
            QueryType::NominationSummary => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        if let Some(validator_details) =
                            self.redis.fetch_validator_details(&account_id)?
                        {
                            log::info!(
                                "Validator selected for nomination summary in chat {}.",
                                chat_id
                            );
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::NominationSummary(validator_details),
                                )
                                .await?;
                        } else {
                            log::warn!(
                                "Validator not found! Selected for nomination summary in chat {}.",
                                chat_id
                            );
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorNotFound(validator_address.clone()),
                                )
                                .await?;
                        }
                    }
                }
            }
            QueryType::NominationDetails => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        if let Some(validator_details) =
                            self.redis.fetch_validator_details(&account_id)?
                        {
                            log::info!(
                                "Validator selected for nomination details in chat {}.",
                                chat_id
                            );
                            let onekv_nominator_account_ids = self
                                .network_postgres
                                .get_onekv_nominator_account_ids()
                                .await?;
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::NominationDetails {
                                        validator_details,
                                        onekv_nominator_account_ids,
                                    },
                                )
                                .await?;
                        } else {
                            log::warn!(
                                "Validator not found! Selected for nomination details in chat {}.",
                                chat_id
                            );
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::ValidatorNotFound(validator_address.clone()),
                                )
                                .await?;
                        }
                    }
                }
            }
            QueryType::RemoveValidator => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        log::info!("Validator selected for removal in chat {}.", chat_id);
                        if self
                            .network_postgres
                            .chat_has_validator(chat_id, &account_id)
                            .await?
                        {
                            if self
                                .network_postgres
                                .remove_validator_from_chat(chat_id, &account_id)
                                .await?
                            {
                                self.update_metrics_validator_count().await?;
                                let app_user_id =
                                    self.network_postgres.get_chat_app_user_id(chat_id).await?;
                                // remove from app, so it doesn't receive notifications
                                let _ = self
                                    .app_postgres
                                    .delete_user_validator_by_account_id(
                                        app_user_id,
                                        CONFIG.substrate.network_id,
                                        &account_id,
                                    )
                                    .await?;
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

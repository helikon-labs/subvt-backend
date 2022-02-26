use crate::{MessageType, TelegramBot};
use serde::{Deserialize, Serialize};
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Balance;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum QueryType {
    #[serde(rename = "V")]
    ValidatorInfo,
    #[serde(rename = "N")]
    Nominations,
    #[serde(rename = "ND")]
    NominationDetails,
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
            QueryType::Nominations => {
                if let Some(validator_address) = &query.parameter {
                    if let Ok(account_id) = AccountId::from_ss58_check(validator_address) {
                        let validator_details = self.redis.fetch_validator_details(&account_id)?;
                        let self_stake = validator_details.self_stake.total_amount;
                        let (
                            active_nominator_count,
                            active_nomination_total,
                            inactive_nominator_count,
                            inactive_nomination_total,
                        ) = if let Some(validator_stake) = &validator_details.validator_stake {
                            let active_nominator_account_ids: Vec<AccountId> = validator_stake
                                .nominators
                                .iter()
                                .map(|n| n.account.id.clone())
                                .collect();
                            let mut inactive_nominator_count: usize = 0;
                            let mut inactive_nomination_total: Balance = 0;
                            for nomination in &validator_details.nominations {
                                if !active_nominator_account_ids
                                    .contains(&nomination.stash_account.id)
                                {
                                    inactive_nominator_count += 1;
                                    inactive_nomination_total += nomination.stake.active_amount;
                                }
                            }
                            (
                                active_nominator_account_ids.len(),
                                validator_stake.total_stake,
                                inactive_nominator_count,
                                inactive_nomination_total,
                            )
                        } else {
                            let inactive_nomination_total: Balance = validator_details
                                .nominations
                                .iter()
                                .map(|n| n.stake.total_amount)
                                .sum();
                            (
                                0,
                                0,
                                validator_details.nominations.len(),
                                inactive_nomination_total,
                            )
                        };
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::NominationSummary {
                                    validator_display: validator_details
                                        .account
                                        .get_display_or_condensed_address()
                                        .clone(),
                                    self_stake,
                                    active_nominator_count,
                                    active_nomination_total,
                                    inactive_nominator_count,
                                    inactive_nomination_total,
                                },
                            )
                            .await?;
                    }
                }
            }
            QueryType::NominationDetails => {}
        }
        Ok(())
    }
}

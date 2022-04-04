use crate::{MessageType, TelegramBot, CONFIG};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use subvt_types::app::{NotificationPeriodType, NotificationTypeCode};

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
    #[serde(rename = "SVA")]
    SettingsValidatorActivity,
    #[serde(rename = "SN")]
    SettingsNominations,
    #[serde(rename = "SKV")]
    SettingsOneKV,
    #[serde(rename = "SD")]
    SettingsDemocracy,
    #[serde(rename = "SE")]
    SettingsEdit(SettingsEditQueryType),
    #[serde(rename = "SB")]
    SettingsBack(SettingsSubSection),
    #[serde(rename = "X")]
    Cancel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SettingsSubSection {
    #[serde(rename = "R")]
    Root,
    #[serde(rename = "VA")]
    ValidatorActivity,
    #[serde(rename = "D")]
    Democracy,
    #[serde(rename = "OKV")]
    OneKV,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SettingsEditQueryType {
    #[serde(rename = "A")]
    Active,
    #[serde(rename = "ANS")]
    ActiveNextSession,
    #[serde(rename = "IA")]
    Inactive,
    #[serde(rename = "IANS")]
    InactiveNextSession,
    #[serde(rename = "CHL")]
    Chilled,
    #[serde(rename = "IC")]
    IdentityChanged,
    #[serde(rename = "OO")]
    OfflineOffence,
    #[serde(rename = "PS")]
    PayoutStakers,
    #[serde(rename = "SKC")]
    SessionKeysChanged,
    #[serde(rename = "SC")]
    SetController,
    #[serde(rename = "UP")]
    UnclaimedPayout,
    #[serde(rename = "DC")]
    DemocracyCancelled,
    #[serde(rename = "DD")]
    DemocracyDelegated,
    #[serde(rename = "DNP")]
    DemocracyNotPassed,
    #[serde(rename = "DP")]
    DemocracyPassed,
    #[serde(rename = "DPR")]
    DemocracyProposed,
    #[serde(rename = "DS")]
    DemocracySeconded,
    #[serde(rename = "DST")]
    DemocracyStarted,
    #[serde(rename = "DU")]
    DemocracyUndelegated,
    #[serde(rename = "DV")]
    DemocracyVoted,
    #[serde(rename = "OKVR")]
    OneKVRankChange,
    #[serde(rename = "OKVB")]
    OneKVBinaryVersionChange,
    #[serde(rename = "OKVV")]
    OneKVValidityChange,
    #[serde(rename = "OKVL")]
    OneKVLocationChange,
}

impl Display for QueryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Self::ValidatorInfo => "ValidatorInfo",
            Self::NominationSummary => "NominationSummary",
            Self::NominationDetails => "NominationDetails",
            Self::RemoveValidator => "RemoveValidator",
            Self::ConfirmBroadcast => "ConfirmBroadcast",
            Self::SettingsValidatorActivity => "SettingsValidatorActivity",
            Self::SettingsNominations => "SettingsNominations",
            Self::SettingsOneKV => "SettingsOneKV",
            Self::SettingsDemocracy => "SettingsDemocracy",
            Self::SettingsEdit(_) => "SettingsEdit",
            Self::SettingsBack(_) => "SettingsBack",
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

impl TelegramBot {
    pub async fn process_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        log::info!("Process query: {}", query);
        crate::metrics::query_call_counter(&query.query_type).inc();
        match query.query_type {
            QueryType::ConfirmBroadcast => {
                if let Some(message_id) = original_message_id {
                    self.messenger.delete_message(chat_id, message_id).await?;
                }
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
                            .fetch_validator_details(&validator.account_id)
                            .await?;
                        if let Some(validator_details) = &maybe_validator_details {
                            self.network_postgres
                                .update_chat_validator_display(
                                    &validator.account_id,
                                    &validator_details.account.get_full_display(),
                                )
                                .await?;
                        }
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::ValidatorInfo {
                                    address: validator.address.clone(),
                                    maybe_validator_details: Box::new(maybe_validator_details),
                                    maybe_onekv_candidate_summary: Box::new(
                                        self.network_postgres
                                            .get_onekv_candidate_summary_by_account_id(
                                                &validator.account_id,
                                            )
                                            .await?,
                                    ),
                                },
                            )
                            .await?;
                    } else {
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::ValidatorNotFound {
                                    maybe_address: None,
                                },
                            )
                            .await?;
                    }
                }
            }
            QueryType::NominationSummary => {
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
                            .fetch_validator_details(&validator.account_id)
                            .await?
                        {
                            log::info!(
                                "Validator selected for nomination summary in chat {}.",
                                chat_id
                            );
                            self.network_postgres
                                .update_chat_validator_display(
                                    &validator.account_id,
                                    &validator_details.account.get_full_display(),
                                )
                                .await?;
                            self.messenger
                                .send_message(
                                    chat_id,
                                    MessageType::NominationSummary {
                                        validator_details,
                                        chat_validator_id: id_str.parse()?,
                                    },
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
                                    MessageType::ValidatorNotFound {
                                        maybe_address: None,
                                    },
                                )
                                .await?;
                        }
                    } else {
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::ValidatorNotFound {
                                    maybe_address: None,
                                },
                            )
                            .await?;
                    }
                }
            }
            QueryType::NominationDetails => {
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
                            .fetch_validator_details(&validator.account_id)
                            .await?
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
                                    MessageType::ValidatorNotFound {
                                        maybe_address: None,
                                    },
                                )
                                .await?;
                        }
                    } else {
                        self.messenger
                            .send_message(
                                chat_id,
                                MessageType::ValidatorNotFound {
                                    maybe_address: None,
                                },
                            )
                            .await?;
                    }
                }
            }
            QueryType::RemoveValidator => {
                if let Some(message_id) = original_message_id {
                    self.messenger.delete_message(chat_id, message_id).await?;
                }
                if let Some(id_str) = &query.parameter {
                    log::info!("Validator selected for removal in chat {}.", chat_id);
                    if let Some(validator) = self
                        .network_postgres
                        .get_chat_validator_by_id(chat_id, id_str.parse()?)
                        .await?
                    {
                        if self
                            .network_postgres
                            .remove_validator_from_chat(chat_id, &validator.account_id)
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
                                    &validator.account_id,
                                )
                                .await?;
                            self.messenger
                                .send_message(chat_id, MessageType::ValidatorRemoved(validator))
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
                                MessageType::ValidatorNotFound {
                                    maybe_address: None,
                                },
                            )
                            .await?;
                    }
                }
            }
            QueryType::SettingsValidatorActivity => {
                let app_user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
                let settings_message_id = match self
                    .network_postgres
                    .get_chat_settings_message_id(chat_id)
                    .await?
                {
                    Some(message_id) => message_id,
                    None => return Ok(()),
                };
                let notification_rules = self
                    .app_postgres
                    .get_user_notification_rules(app_user_id)
                    .await?;
                self.messenger
                    .update_settings_message(
                        chat_id,
                        settings_message_id,
                        &SettingsSubSection::ValidatorActivity,
                        &notification_rules,
                    )
                    .await?;
            }
            QueryType::SettingsNominations => (),
            QueryType::SettingsDemocracy => {
                let app_user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
                let settings_message_id = match self
                    .network_postgres
                    .get_chat_settings_message_id(chat_id)
                    .await?
                {
                    Some(message_id) => message_id,
                    None => return Ok(()),
                };
                let notification_rules = self
                    .app_postgres
                    .get_user_notification_rules(app_user_id)
                    .await?;
                self.messenger
                    .update_settings_message(
                        chat_id,
                        settings_message_id,
                        &SettingsSubSection::Democracy,
                        &notification_rules,
                    )
                    .await?;
            }
            QueryType::SettingsOneKV => {
                let app_user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
                let settings_message_id = match self
                    .network_postgres
                    .get_chat_settings_message_id(chat_id)
                    .await?
                {
                    Some(message_id) => message_id,
                    None => return Ok(()),
                };
                let notification_rules = self
                    .app_postgres
                    .get_user_notification_rules(app_user_id)
                    .await?;
                self.messenger
                    .update_settings_message(
                        chat_id,
                        settings_message_id,
                        &SettingsSubSection::OneKV,
                        &notification_rules,
                    )
                    .await?;
            }
            QueryType::SettingsEdit(ref edit_query_type) => {
                let user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
                let sub_section = match edit_query_type {
                    SettingsEditQueryType::Active => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator active notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorActive,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::ActiveNextSession => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator active next session notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorActiveNextSession,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::Inactive => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator inactive notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorInactive,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::InactiveNextSession => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator inactive next session notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorInactiveNextSession,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::Chilled => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator chilled notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorChilled,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::IdentityChanged => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator identity changed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorIdentityChanged,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::OfflineOffence => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator offline offence notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorOfflineOffence,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::PayoutStakers => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator payout stakers notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorPayoutStakers,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::SessionKeysChanged => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator session keys changed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorSessionKeysChanged,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::SetController => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator controller changed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorSetController,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::UnclaimedPayout => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for validator unclaimed payout notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::ChainValidatorUnclaimedPayout,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::ValidatorActivity
                    }
                    SettingsEditQueryType::DemocracyCancelled => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy cancelled notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyCancelled,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracyDelegated => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy delegated notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyDelegated,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracyNotPassed => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy not passed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyNotPassed,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracyPassed => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy passed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyPassed,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracyProposed => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy passed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyProposed,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracySeconded => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy passed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracySeconded,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracyStarted => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy passed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyStarted,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracyUndelegated => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy passed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyUndelegated,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::DemocracyVoted => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for democracy passed notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::DemocracyVoted,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::Democracy
                    }
                    SettingsEditQueryType::OneKVRankChange => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for 1KV rank change notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::OneKVValidatorRankChange,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::OneKV
                    }
                    SettingsEditQueryType::OneKVBinaryVersionChange => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for 1KV binary version change notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::OneKVValidatorBinaryVersionChange,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::OneKV
                    }
                    SettingsEditQueryType::OneKVValidityChange => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for 1KV validity change notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::OneKVValidatorValidityChange,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::OneKV
                    }
                    SettingsEditQueryType::OneKVLocationChange => {
                        let on: bool = serde_json::from_str(
                            query.parameter.as_ref().unwrap_or_else(||
                                panic!("Expecting on/off param for 1KV location change notification setting action.")
                            )
                        )?;
                        self.app_postgres
                            .update_user_notification_rule_period(
                                user_id,
                                &NotificationTypeCode::OneKVValidatorLocationChange,
                                if on {
                                    &NotificationPeriodType::Immediate
                                } else {
                                    &NotificationPeriodType::Off
                                },
                                0,
                            )
                            .await?;
                        SettingsSubSection::OneKV
                    }
                };
                let notification_rules = self
                    .app_postgres
                    .get_user_notification_rules(user_id)
                    .await?;
                if let Some(settings_message_id) = self
                    .network_postgres
                    .get_chat_settings_message_id(chat_id)
                    .await?
                {
                    self.messenger
                        .update_settings_message(
                            chat_id,
                            settings_message_id,
                            &sub_section,
                            &notification_rules,
                        )
                        .await?;
                }
            }
            QueryType::SettingsBack(ref sub_type) => {
                if let Some(settings_message_id) = self
                    .network_postgres
                    .get_chat_settings_message_id(chat_id)
                    .await?
                {
                    let user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
                    let notification_rules = self
                        .app_postgres
                        .get_user_notification_rules(user_id)
                        .await?;
                    self.messenger
                        .update_settings_message(
                            chat_id,
                            settings_message_id,
                            sub_type,
                            &notification_rules,
                        )
                        .await?;
                }
            }
            QueryType::Cancel => {
                if let Some(message_id) = original_message_id {
                    self.messenger.delete_message(chat_id, message_id).await?;
                }
            }
        }
        Ok(())
    }
}

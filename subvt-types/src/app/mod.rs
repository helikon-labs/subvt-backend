use crate::crypto::AccountId;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub mod db;
pub mod event;
pub mod extrinsic;

pub const PUBLIC_KEY_HEX_LENGTH: usize = 64;

pub struct Block {
    pub hash: String,
    pub number: u64,
    pub timestamp: Option<u64>,
    pub author_account_id: Option<AccountId>,
    pub era_index: u64,
    pub epoch_index: u64,
    pub is_finalized: bool,
    pub metadata_version: u16,
    pub runtime_version: u16,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Network {
    pub id: u32,
    pub hash: String,
    pub name: String,
    pub ss58_prefix: u32,
    pub live_network_status_service_url: Option<String>,
    pub report_service_url: Option<String>,
    pub validator_details_service_url: Option<String>,
    pub validator_list_service_url: Option<String>,
}

fn default_id() -> u32 {
    0
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    #[serde(default = "default_id")]
    pub id: u32,
    pub public_key_hex: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NotificationChannel {
    pub code: String,
}

pub enum NotificationTypeCode {
    ChainValidatorOfflineOffence,
    ChainValidatorNewNomination,
    ChainValidatorLostNomination,
    ChainValidatorChilled,
    ChainValidatorActiveSetInclusion,
    ChainValidatorActiveSetNextSessionInclusion,
    ChainValidatorActiveSetExclusion,
    ChainValidatorActiveSetNextSessionExclusion,
    ChainValidateExtrinsic,
    ChainValidatorUnclaimedPayout,
    ChainValidatorBlockAuthorship,
    TelemetryValidatorOffline,
    TelemetryValidatorBinaryOutOfDate,
    TelemetryValidatorPeerCountLow,
    TelemetryValidatorTooManyTxsInQueue,
    TelemetryValidatorLagging,
    TelemetryValidatorFinalityLagging,
    TelemetryValidatorDownloadBwLow,
    TelemetryValidatorUploadBwLow,
    OneKVValidatorRankChange,
    OneKVValidatorValidityChange,
}

impl Display for NotificationTypeCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            NotificationTypeCode::ChainValidatorOfflineOffence => "chain_validator_offline_offence",
            NotificationTypeCode::ChainValidatorNewNomination => "chain_validator_new_nomination",
            NotificationTypeCode::ChainValidatorLostNomination => "chain_validator_lost_nomination",
            NotificationTypeCode::ChainValidatorChilled => "chain_validator_chilled",
            NotificationTypeCode::ChainValidatorActiveSetInclusion => {
                "chain_validator_active_set_inclusion"
            }
            NotificationTypeCode::ChainValidatorActiveSetNextSessionInclusion => {
                "chain_validator_active_set_next_session_inclusion"
            }
            NotificationTypeCode::ChainValidatorActiveSetExclusion => {
                "chain_validator_active_set_exclusion"
            }
            NotificationTypeCode::ChainValidatorActiveSetNextSessionExclusion => {
                "chain_validator_active_set_next_session_exclusion"
            }
            NotificationTypeCode::ChainValidateExtrinsic => "chain_validate_extrinsic",
            NotificationTypeCode::ChainValidatorUnclaimedPayout => {
                "chain_validator_unclaimed_payout"
            }
            NotificationTypeCode::ChainValidatorBlockAuthorship => {
                "chain_validator_block_authorship"
            }
            NotificationTypeCode::TelemetryValidatorOffline => "telemetry_validator_offline",
            NotificationTypeCode::TelemetryValidatorBinaryOutOfDate => {
                "telemetry_validator_binary_out_of_date"
            }
            NotificationTypeCode::TelemetryValidatorPeerCountLow => {
                "telemetry_validator_peer_count_low"
            }
            NotificationTypeCode::TelemetryValidatorTooManyTxsInQueue => {
                "telemetry_validator_too_many_txs_in_queue"
            }
            NotificationTypeCode::TelemetryValidatorLagging => "telemetry_validator_lagging",
            NotificationTypeCode::TelemetryValidatorFinalityLagging => {
                "telemetry_validator_finality_lagging"
            }
            NotificationTypeCode::TelemetryValidatorDownloadBwLow => {
                "telemetry_validator_download_bw_low"
            }
            NotificationTypeCode::TelemetryValidatorUploadBwLow => {
                "telemetry_validator_upload_bw_low"
            }
            NotificationTypeCode::OneKVValidatorRankChange => "onekv_validator_rank_change",
            NotificationTypeCode::OneKVValidatorValidityChange => "onekv_validator_validity_change",
        };
        write!(f, "{}", code)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NotificationType {
    pub code: String,
    pub param_types: Vec<NotificationParamType>,
}

#[derive(Clone, Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(
    type_name = "app_notification_type_param_data_type",
    rename_all = "lowercase"
)]
#[serde(rename_all = "lowercase")]
pub enum NotificationParamDataType {
    String,
    Integer,
    Balance,
    Float,
    Boolean,
}

impl Display for NotificationParamDataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NotificationParamDataType::String => "string",
                NotificationParamDataType::Integer => "integer",
                NotificationParamDataType::Balance => "balance",
                NotificationParamDataType::Float => "float",
                NotificationParamDataType::Boolean => "boolean",
            }
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotificationParamType {
    pub id: u32,
    pub notification_type_code: String,
    pub order: u8,
    pub code: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_: NotificationParamDataType,
    pub min: Option<String>,
    pub max: Option<String>,
    pub is_optional: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserNotificationChannel {
    #[serde(default = "default_id")]
    pub id: u32,
    #[serde(default = "default_id")]
    pub user_id: u32,
    pub channel_code: String,
    pub target: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserValidator {
    #[serde(default = "default_id")]
    pub id: u32,
    #[serde(default = "default_id")]
    pub user_id: u32,
    pub network_id: u32,
    pub validator_account_id: AccountId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserNotificationRuleParameter {
    #[serde(default = "default_id")]
    pub user_notification_rule_id: u32,
    pub parameter_type_id: u32,
    #[serde(default)]
    pub parameter_type_code: String,
    #[serde(default)]
    pub order: u8,
    pub value: String,
}

impl From<&(i32, i32, String, i16, String)> for UserNotificationRuleParameter {
    fn from(input: &(i32, i32, String, i16, String)) -> Self {
        UserNotificationRuleParameter {
            user_notification_rule_id: input.0 as u32,
            parameter_type_id: input.1 as u32,
            parameter_type_code: input.2.clone(),
            order: input.3 as u8,
            value: input.4.clone(),
        }
    }
}

impl UserNotificationRuleParameter {
    pub fn validate(&self, parameter_type: &NotificationParamType) -> (bool, Option<String>) {
        match parameter_type.type_ {
            NotificationParamDataType::String => {
                if let Some(min) = &parameter_type.min {
                    if self.value.len() < min.parse::<usize>().unwrap() {
                        return (
                            false,
                            Some(format!("String length cannot be less than {}.", min)),
                        );
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if self.value.len() > max.parse::<usize>().unwrap() {
                        return (
                            false,
                            Some(format!("String length cannot be more than {}.", max)),
                        );
                    }
                }
            }
            NotificationParamDataType::Integer => {
                if let Some(min) = &parameter_type.min {
                    if let Ok(value) = self.value.parse::<i64>() {
                        if value < min.parse::<i64>().unwrap() {
                            return (false, Some(format!("Cannot be less than {}.", min)));
                        }
                    } else {
                        return (false, Some("Invalid integer value.".to_string()));
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if let Ok(value) = self.value.parse::<i64>() {
                        if value > max.parse::<i64>().unwrap() {
                            return (false, Some(format!("Cannot be more than {}.", max)));
                        }
                    } else {
                        return (false, Some("Invalid integer value.".to_string()));
                    }
                }
            }
            NotificationParamDataType::Balance => {
                if let Some(min) = &parameter_type.min {
                    if let Ok(value) = self.value.parse::<u128>() {
                        if value < min.parse::<u128>().unwrap() {
                            return (false, Some(format!("Cannot be less than {}.", min)));
                        }
                    } else {
                        return (false, Some("Invalid balance value.".to_string()));
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if let Ok(value) = self.value.parse::<u128>() {
                        if value > max.parse::<u128>().unwrap() {
                            return (false, Some(format!("Cannot be more than {}.", max)));
                        }
                    } else {
                        return (false, Some("Invalid balance value.".to_string()));
                    }
                }
            }
            NotificationParamDataType::Float => {
                if let Some(min) = &parameter_type.min {
                    if let Ok(value) = self.value.parse::<f64>() {
                        if value < min.parse::<f64>().unwrap() {
                            return (false, Some(format!("Cannot be less than {}.", min)));
                        }
                    } else {
                        return (false, Some("Invalid float value.".to_string()));
                    }
                }
                if let Some(max) = &parameter_type.max {
                    if let Ok(value) = self.value.parse::<f64>() {
                        if value > max.parse::<f64>().unwrap() {
                            return (false, Some(format!("Cannot be more than {}.", max)));
                        }
                    } else {
                        return (false, Some("Invalid float value.".to_string()));
                    }
                }
            }
            NotificationParamDataType::Boolean => {
                if self.value.parse::<bool>().is_err() {
                    return (false, Some("Invalid boolean value.".to_string()));
                }
            }
        }
        (true, None)
    }
}

#[derive(Clone, Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "app_notification_period_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum NotificationPeriodType {
    Immediate,
    Hour,
    Day,
    Epoch,
    Session,
    Era,
}

impl Display for NotificationPeriodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NotificationPeriodType::Immediate => "immediate",
                NotificationPeriodType::Hour => "hour",
                NotificationPeriodType::Day => "day",
                NotificationPeriodType::Epoch => "epoch",
                NotificationPeriodType::Session => "session",
                NotificationPeriodType::Era => "era",
            }
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UserNotificationRule {
    pub id: u32,
    pub user_id: u32,
    pub notification_type: NotificationType,
    pub name: Option<String>,
    pub network: Option<Network>,
    pub is_for_all_validators: bool,
    pub period_type: NotificationPeriodType,
    pub period: u16,
    pub validators: Vec<UserValidator>,
    pub notification_channels: Vec<UserNotificationChannel>,
    pub parameters: Vec<UserNotificationRuleParameter>,
    pub notes: Option<String>,
}

pub struct Notification {
    pub id: u32,
    pub user_id: u32,
    pub user_notification_rule_id: u32,
    pub network_id: u32,
    pub period_type: NotificationPeriodType,
    pub period: u16,
    pub validator_account_id: AccountId,
    pub notification_type_code: String,
    pub parameter_type_id: Option<u32>,
    pub parameter_value: Option<String>,
    pub block_hash: Option<String>,
    pub block_number: Option<u64>,
    pub block_timestamp: Option<u64>,
    pub extrinsic_index: Option<u32>,
    pub event_index: Option<u32>,
    pub user_notification_channel_id: u32,
    pub notification_channel_code: String,
    pub notification_target: String,
    pub log: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub sent_at: Option<NaiveDateTime>,
    pub delivered_at: Option<NaiveDateTime>,
    pub read_at: Option<NaiveDateTime>,
}

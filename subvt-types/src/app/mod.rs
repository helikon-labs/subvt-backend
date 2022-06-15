//! Types used in the application logic of SubVT.
use crate::crypto::AccountId;
use crate::substrate::Account;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub mod app_event;
pub mod db;
pub mod event;
pub mod extrinsic;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Network {
    pub id: u32,
    pub hash: String,
    pub chain: String,
    pub display: String,
    pub ss58_prefix: u32,
    pub token_ticker: String,
    pub token_decimal_count: u8,
    pub network_status_service_host: Option<String>,
    pub network_status_service_port: Option<u16>,
    pub report_service_host: Option<String>,
    pub report_service_port: Option<u16>,
    pub validator_details_service_host: Option<String>,
    pub validator_details_service_port: Option<u16>,
    pub active_validator_list_service_host: Option<String>,
    pub active_validator_list_service_port: Option<u16>,
    pub inactive_validator_list_service_host: Option<String>,
    pub inactive_validator_list_service_port: Option<u16>,
}

fn default_id() -> u32 {
    0
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct User {
    #[serde(default = "default_id")]
    pub id: u32,
    pub public_key_hex: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum NotificationChannel {
    #[serde(rename = "apns")]
    APNS,
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "fcm")]
    FCM,
    #[serde(rename = "gsm")]
    GSM,
    #[serde(rename = "telegram")]
    Telegram,
    #[serde(rename = "sms")]
    SMS,
}

impl Display for NotificationChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::APNS => "apns",
            Self::Email => "email",
            Self::FCM => "fcm",
            Self::GSM => "gsm",
            Self::Telegram => "telegram",
            Self::SMS => "sms",
        };
        write!(f, "{}", str)
    }
}

impl From<&str> for NotificationChannel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "apns" => Self::APNS,
            "email" => Self::Email,
            "fcm" => Self::FCM,
            "gsm" => Self::GSM,
            "telegram" => Self::Telegram,
            "sms" => Self::SMS,
            _ => panic!("Unkown chain: {}", s),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationTypeCode {
    ChainValidateExtrinsic,
    ChainValidatorActive,
    ChainValidatorActiveNextSession,
    ChainValidatorBlockAuthorship,
    ChainValidatorChilled,
    ChainValidatorIdentityChanged,
    ChainValidatorInactive,
    ChainValidatorInactiveNextSession,
    ChainValidatorLostNomination,
    ChainValidatorNewNomination,
    ChainValidatorNominationAmountChange,
    ChainValidatorOfflineOffence,
    ChainValidatorPayoutStakers,
    ChainValidatorSessionKeysChanged,
    ChainValidatorSetController,
    ChainValidatorUnclaimedPayout,
    OneKVValidatorBinaryVersionChange,
    OneKVValidatorLocationChange,
    OneKVValidatorRankChange,
    OneKVValidatorValidityChange,
    OneKVValidatorOnlineStatusChange,
    TelemetryValidatorBinaryOutOfDate,
    TelemetryValidatorDownloadBwLow,
    TelemetryValidatorFinalityLagging,
    TelemetryValidatorLagging,
    TelemetryValidatorOffline,
    TelemetryValidatorPeerCountLow,
    TelemetryValidatorTooManyTxsInQueue,
    TelemetryValidatorUploadBwLow,
    DemocracyCancelled,
    DemocracyDelegated,
    DemocracyNotPassed,
    DemocracyPassed,
    DemocracyProposed,
    DemocracySeconded,
    DemocracyStarted,
    DemocracyUndelegated,
    DemocracyVoted,
}

impl Display for NotificationTypeCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            NotificationTypeCode::ChainValidatorOfflineOffence => "chain_validator_offline_offence",
            NotificationTypeCode::ChainValidatorNewNomination => "chain_validator_new_nomination",
            NotificationTypeCode::ChainValidatorLostNomination => "chain_validator_lost_nomination",
            NotificationTypeCode::ChainValidatorNominationAmountChange => {
                "chain_validator_nomination_amount_change"
            }
            NotificationTypeCode::ChainValidatorChilled => "chain_validator_chilled",
            NotificationTypeCode::ChainValidatorActive => "chain_validator_active",
            NotificationTypeCode::ChainValidatorActiveNextSession => {
                "chain_validator_active_next_session"
            }
            NotificationTypeCode::ChainValidatorInactive => "chain_validator_inactive",
            NotificationTypeCode::ChainValidatorInactiveNextSession => {
                "chain_validator_inactive_next_session"
            }
            NotificationTypeCode::ChainValidateExtrinsic => "chain_validate_extrinsic",
            NotificationTypeCode::ChainValidatorUnclaimedPayout => {
                "chain_validator_unclaimed_payout"
            }
            NotificationTypeCode::ChainValidatorBlockAuthorship => {
                "chain_validator_block_authorship"
            }
            NotificationTypeCode::ChainValidatorSetController => "chain_validator_set_controller",
            NotificationTypeCode::ChainValidatorSessionKeysChanged => {
                "chain_validator_session_keys_changed"
            }
            NotificationTypeCode::ChainValidatorIdentityChanged => {
                "chain_validator_identity_changed"
            }
            NotificationTypeCode::ChainValidatorPayoutStakers => "chain_validator_payout_stakers",
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
            NotificationTypeCode::OneKVValidatorBinaryVersionChange => {
                "onekv_validator_binary_version_change"
            }
            NotificationTypeCode::OneKVValidatorRankChange => "onekv_validator_rank_change",
            NotificationTypeCode::OneKVValidatorLocationChange => "onekv_validator_location_change",
            NotificationTypeCode::OneKVValidatorValidityChange => "onekv_validator_validity_change",
            NotificationTypeCode::OneKVValidatorOnlineStatusChange => {
                "onekv_validator_online_status_change"
            }
            // democracy
            NotificationTypeCode::DemocracyCancelled => "democracy_cancelled",
            NotificationTypeCode::DemocracyDelegated => "democracy_delegated",
            NotificationTypeCode::DemocracyNotPassed => "democracy_not_passed",
            NotificationTypeCode::DemocracyPassed => "democracy_passed",
            NotificationTypeCode::DemocracyProposed => "democracy_proposed",
            NotificationTypeCode::DemocracySeconded => "democracy_seconded",
            NotificationTypeCode::DemocracyStarted => "democracy_started",
            NotificationTypeCode::DemocracyUndelegated => "democracy_undelegated",
            NotificationTypeCode::DemocracyVoted => "democracy_voted",
        };
        write!(f, "{}", code)
    }
}

impl From<&str> for NotificationTypeCode {
    fn from(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "chain_validator_offline_offence" => NotificationTypeCode::ChainValidatorOfflineOffence,
            "chain_validator_new_nomination" => NotificationTypeCode::ChainValidatorNewNomination,
            "chain_validator_lost_nomination" => NotificationTypeCode::ChainValidatorLostNomination,
            "chain_validator_nomination_amount_change" => {
                NotificationTypeCode::ChainValidatorNominationAmountChange
            }
            "chain_validator_chilled" => NotificationTypeCode::ChainValidatorChilled,
            "chain_validator_active" => NotificationTypeCode::ChainValidatorActive,
            "chain_validator_active_next_session" => {
                NotificationTypeCode::ChainValidatorActiveNextSession
            }
            "chain_validator_inactive" => NotificationTypeCode::ChainValidatorInactive,
            "chain_validator_inactive_next_session" => {
                NotificationTypeCode::ChainValidatorInactiveNextSession
            }
            "chain_validate_extrinsic" => NotificationTypeCode::ChainValidateExtrinsic,
            "chain_validator_unclaimed_payout" => {
                NotificationTypeCode::ChainValidatorUnclaimedPayout
            }
            "chain_validator_block_authorship" => {
                NotificationTypeCode::ChainValidatorBlockAuthorship
            }
            "chain_validator_set_controller" => NotificationTypeCode::ChainValidatorSetController,
            "chain_validator_session_keys_changed" => {
                NotificationTypeCode::ChainValidatorSessionKeysChanged
            }
            "chain_validator_identity_changed" => {
                NotificationTypeCode::ChainValidatorIdentityChanged
            }
            "chain_validator_payout_stakers" => NotificationTypeCode::ChainValidatorPayoutStakers,
            "telemetry_validator_offline" => NotificationTypeCode::TelemetryValidatorOffline,
            "telemetry_validator_binary_out_of_date" => {
                NotificationTypeCode::TelemetryValidatorBinaryOutOfDate
            }
            "telemetry_validator_peer_count_low" => {
                NotificationTypeCode::TelemetryValidatorPeerCountLow
            }
            "telemetry_validator_too_many_txs_in_queue" => {
                NotificationTypeCode::TelemetryValidatorTooManyTxsInQueue
            }
            "telemetry_validator_lagging" => NotificationTypeCode::TelemetryValidatorLagging,
            "telemetry_validator_finality_lagging" => {
                NotificationTypeCode::TelemetryValidatorFinalityLagging
            }
            "telemetry_validator_download_bw_low" => {
                NotificationTypeCode::TelemetryValidatorDownloadBwLow
            }
            "telemetry_validator_upload_bw_low" => {
                NotificationTypeCode::TelemetryValidatorUploadBwLow
            }
            "onekv_validator_binary_version_change" => {
                NotificationTypeCode::OneKVValidatorBinaryVersionChange
            }
            "onekv_validator_rank_change" => NotificationTypeCode::OneKVValidatorRankChange,
            "onekv_validator_location_change" => NotificationTypeCode::OneKVValidatorLocationChange,
            "onekv_validator_validity_change" => NotificationTypeCode::OneKVValidatorValidityChange,
            "onekv_validator_online_status_change" => {
                NotificationTypeCode::OneKVValidatorOnlineStatusChange
            }
            "democracy_cancelled" => NotificationTypeCode::DemocracyCancelled,
            "democracy_delegated" => NotificationTypeCode::DemocracyDelegated,
            "democracy_not_passed" => NotificationTypeCode::DemocracyNotPassed,
            "democracy_passed" => NotificationTypeCode::DemocracyPassed,
            "democracy_proposed" => NotificationTypeCode::DemocracyProposed,
            "democracy_seconded" => NotificationTypeCode::DemocracySeconded,
            "democracy_started" => NotificationTypeCode::DemocracyStarted,
            "democracy_undelegated" => NotificationTypeCode::DemocracyUndelegated,
            "democracy_voted" => NotificationTypeCode::DemocracyVoted,
            _ => panic!("Unknown notification type code: {}", code),
        }
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserNotificationChannel {
    #[serde(default = "default_id")]
    pub id: u32,
    #[serde(default = "default_id")]
    pub user_id: u32,
    pub channel: NotificationChannel,
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

#[derive(Clone, Copy, Debug, sqlx::Type, Serialize, Deserialize, Eq, PartialEq)]
#[sqlx(type_name = "app_notification_period_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum NotificationPeriodType {
    Off,
    Immediate,
    Hour,
    Day,
    Epoch,
    Era,
}

impl Display for NotificationPeriodType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NotificationPeriodType::Off => "off",
                NotificationPeriodType::Immediate => "immediate",
                NotificationPeriodType::Hour => "hour",
                NotificationPeriodType::Day => "day",
                NotificationPeriodType::Epoch => "epoch",
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

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: u32,
    pub user_id: u32,
    pub user_notification_rule_id: u32,
    pub network_id: u32,
    pub period_type: NotificationPeriodType,
    pub period: u16,
    pub validator_account_id: Option<AccountId>,
    pub validator_account_json: Option<String>,
    pub notification_type_code: String,
    pub user_notification_channel_id: u32,
    pub notification_channel: NotificationChannel,
    pub notification_target: String,
    pub data_json: Option<String>,
    pub log: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub sent_at: Option<NaiveDateTime>,
    pub delivered_at: Option<NaiveDateTime>,
    pub read_at: Option<NaiveDateTime>,
}

impl Notification {
    pub fn get_account(&self) -> anyhow::Result<Option<Account>> {
        if let Some(account_json) = &self.validator_account_json {
            Ok(Some(serde_json::from_str(account_json)?))
        } else {
            Ok(None)
        }
    }
}

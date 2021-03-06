use crate::crypto::AccountId;
use crate::subvt::{NominationSummary, ValidatorDetails};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use subvt_utility::numeric::format_decimal;

#[derive(Debug, Eq, PartialEq)]
pub enum TelegramChatState {
    Default,
    AddValidator,
    EnterBugReport,
    EnterFeatureRequest,
    EnterMigrationCode,
}

impl FromStr for TelegramChatState {
    type Err = std::string::ParseError;

    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "Default" => Ok(Self::Default),
            "AddValidator" => Ok(Self::AddValidator),
            "EnterBugReport" => Ok(Self::EnterBugReport),
            "EnterFeatureRequest" => Ok(Self::EnterFeatureRequest),
            "EnterMigrationCode" => Ok(Self::EnterMigrationCode),
            _ => panic!("Unknown state: {}", state),
        }
    }
}

impl Display for TelegramChatState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            TelegramChatState::Default => "Default",
            TelegramChatState::AddValidator => "AddValidator",
            TelegramChatState::EnterBugReport => "EnterBugReport",
            TelegramChatState::EnterFeatureRequest => "EnterFeatureRequest",
            TelegramChatState::EnterMigrationCode => "EnterMigrationCode",
        };
        write!(f, "{}", display)
    }
}

#[derive(Debug)]
pub struct TelegramChatValidator {
    pub id: u64,
    pub account_id: AccountId,
    pub address: String,
    pub display: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TelegramChatValidatorSummary {
    pub display: String,
    pub address: String,
    pub is_active: bool,
    pub active_next_session: bool,
    pub is_para_validator: bool,
    pub is_valid_onekv: Option<bool>,
    pub self_stake_formatted: String,
    pub inactive_nomination_formatted: String,
    pub inactive_nominator_count: usize,
    pub active_nomination_formatted: String,
    pub active_nominator_count: usize,
    pub heartbeat_received: bool,
    pub missing_referendum_votes: Vec<u32>,
}

impl TelegramChatValidatorSummary {
    pub fn from(
        validator_details: &ValidatorDetails,
        token_decimals: usize,
        token_format_decimal_points: usize,
    ) -> TelegramChatValidatorSummary {
        let self_stake_formatted = format_decimal(
            validator_details.self_stake.total_amount,
            token_decimals,
            token_format_decimal_points,
        );
        let nomination_summary: NominationSummary = validator_details.into();
        let active_nomination_formatted = format_decimal(
            nomination_summary.active_nomination_total,
            token_decimals,
            token_format_decimal_points,
        );
        let inactive_nomination_formatted = format_decimal(
            nomination_summary.inactive_nomination_total,
            token_decimals,
            token_format_decimal_points,
        );
        let display = validator_details
            .account
            .get_display_or_condensed_address(None);
        TelegramChatValidatorSummary {
            display,
            address: validator_details.account.address.clone(),
            is_active: validator_details.is_active,
            active_next_session: validator_details.active_next_session,
            is_para_validator: validator_details.is_para_validator,
            is_valid_onekv: validator_details.onekv_is_valid,
            self_stake_formatted,
            inactive_nomination_formatted,
            inactive_nominator_count: nomination_summary.inactive_nominator_count,
            active_nomination_formatted,
            active_nominator_count: nomination_summary.active_nominator_count,
            heartbeat_received: validator_details.heartbeat_received.unwrap_or(false),
            missing_referendum_votes: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVBotChat {
    pub chat_id: i64,
    pub block_notification_period: i64,
    pub unclaimed_payout_notification_period: i64,
    pub send_new_nomination_notifications: bool,
    pub send_chilling_event_notifications: bool,
    pub send_offline_event_notifications: bool,
    pub migration_code: String,
    pub is_migrated: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneKVBotValidator {
    pub stash_address: String,
    pub name: String,
    pub chat_ids: Vec<i64>,
}

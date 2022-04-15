use crate::crypto::AccountId;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum TelegramChatState {
    Default,
    AddValidator,
    EnterBugReport,
    EnterFeatureRequest,
}

impl FromStr for TelegramChatState {
    type Err = std::string::ParseError;

    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "Default" => Ok(Self::Default),
            "AddValidator" => Ok(Self::AddValidator),
            "EnterBugReport" => Ok(Self::EnterBugReport),
            "EnterFeatureRequest" => Ok(Self::EnterFeatureRequest),
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

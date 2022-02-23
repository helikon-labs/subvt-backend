use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum TelegramChatState {
    Default,
    AddValidator,
}

impl FromStr for TelegramChatState {
    type Err = std::string::ParseError;

    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "Default" => Ok(Self::Default),
            "AddValidator" => Ok(Self::AddValidator),
            _ => panic!("Unknown state: {}", state),
        }
    }
}

impl Display for TelegramChatState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            TelegramChatState::Default => "Default",
            TelegramChatState::AddValidator => "AddValidator",
        };
        write!(f, "{}", display)
    }
}

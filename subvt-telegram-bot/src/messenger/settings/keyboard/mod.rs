use crate::{
    query::{QueryType, SettingsSubSection},
    Messenger,
};
use frankenstein::InlineKeyboardMarkup;

pub(crate) mod active_inactive;
pub(crate) mod democracy;
pub(crate) mod nomination;
pub(crate) mod onekv;
pub(crate) mod period;
pub(crate) mod validator_activity;

impl Messenger {
    pub fn get_settings_keyboard(&self) -> anyhow::Result<InlineKeyboardMarkup> {
        let rows = vec![
            self.get_settings_button("settings_root_title.html", QueryType::NoOp)?,
            self.get_settings_button(
                "settings_validator_activity.html",
                QueryType::SettingsNavigate(SettingsSubSection::ValidatorActivity),
            )?,
            self.get_settings_button(
                "settings_nominations.html",
                QueryType::SettingsNavigate(SettingsSubSection::Nominations),
            )?,
            self.get_settings_button(
                "settings_democracy.html",
                QueryType::SettingsNavigate(SettingsSubSection::Democracy),
            )?,
            self.get_settings_button(
                "settings_onekv.html",
                QueryType::SettingsNavigate(SettingsSubSection::OneKV),
            )?,
            self.get_settings_button("cancel.html", QueryType::Cancel)?,
        ];
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }
}

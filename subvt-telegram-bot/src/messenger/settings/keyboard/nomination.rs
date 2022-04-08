use super::Messenger;
use crate::query::{QueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;

impl Messenger {
    pub(crate) fn get_nomination_settings_keyboard(&self) -> anyhow::Result<InlineKeyboardMarkup> {
        let rows = vec![
            self.get_settings_button("settings_nominations_title.html", QueryType::NoOp)?,
            self.get_settings_button(
                "settings_new_nominations.html",
                QueryType::SettingsNavigate(SettingsSubSection::NewNomination),
            )?,
            self.get_settings_button(
                "settings_lost_nominations.html",
                QueryType::SettingsNavigate(SettingsSubSection::LostNomination),
            )?,
            self.get_settings_button(
                "back.html",
                QueryType::SettingsNavigate(SettingsSubSection::Root),
            )?,
            self.get_settings_button("cancel.html", QueryType::Cancel)?,
        ];
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }
}

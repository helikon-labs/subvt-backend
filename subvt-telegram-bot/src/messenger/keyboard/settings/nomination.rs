use crate::messenger::button::settings::get_settings_button;
use crate::query::{QueryType, SettingsSubSection};
use frankenstein::types::InlineKeyboardMarkup;
use tera::Tera;

pub(crate) fn get_nomination_settings_keyboard(
    renderer: &Tera,
) -> anyhow::Result<InlineKeyboardMarkup> {
    let rows = vec![
        get_settings_button(renderer, "settings_nominations_title.html", QueryType::NoOp)?,
        get_settings_button(
            renderer,
            "settings_new_nominations.html",
            QueryType::SettingsNavigate(SettingsSubSection::NewNomination),
        )?,
        get_settings_button(
            renderer,
            "settings_lost_nominations.html",
            QueryType::SettingsNavigate(SettingsSubSection::LostNomination),
        )?,
        get_settings_button(
            renderer,
            "back.html",
            QueryType::SettingsNavigate(SettingsSubSection::Root),
        )?,
        get_settings_button(renderer, "close.html", QueryType::Close)?,
    ];
    Ok(InlineKeyboardMarkup {
        inline_keyboard: rows,
    })
}

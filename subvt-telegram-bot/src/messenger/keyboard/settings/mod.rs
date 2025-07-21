use crate::messenger::button::settings::get_settings_button;
use crate::query::{QueryType, SettingsSubSection};
use frankenstein::types::InlineKeyboardMarkup;
use tera::Tera;

pub(crate) mod active_inactive;
pub(crate) mod nomination;
pub(crate) mod onekv;
pub(crate) mod para_validation;
pub(crate) mod period;
pub(crate) mod referenda;
pub(crate) mod validator_activity;

pub fn get_settings_keyboard(renderer: &Tera) -> anyhow::Result<InlineKeyboardMarkup> {
    let rows = vec![
        get_settings_button(renderer, "settings_root_title.html", QueryType::NoOp)?,
        get_settings_button(
            renderer,
            "settings_validator_activity.html",
            QueryType::SettingsNavigate(SettingsSubSection::ValidatorActivity),
        )?,
        get_settings_button(
            renderer,
            "settings_nominations.html",
            QueryType::SettingsNavigate(SettingsSubSection::Nominations),
        )?,
        get_settings_button(
            renderer,
            "settings_referenda.html",
            QueryType::SettingsNavigate(SettingsSubSection::Referenda),
        )?,
        get_settings_button(
            renderer,
            "settings_onekv.html",
            QueryType::SettingsNavigate(SettingsSubSection::OneKV),
        )?,
        get_settings_button(renderer, "close.html", QueryType::Close)?,
    ];
    Ok(InlineKeyboardMarkup {
        inline_keyboard: rows,
    })
}

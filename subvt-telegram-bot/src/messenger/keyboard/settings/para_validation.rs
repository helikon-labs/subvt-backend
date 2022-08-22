use crate::messenger::button::settings::{get_notification_on_off_button, get_settings_button};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};
use tera::Tera;

pub(crate) fn get_para_validation_settings_keyboard(
    renderer: &Tera,
    notification_rules: &[UserNotificationRule],
) -> anyhow::Result<InlineKeyboardMarkup> {
    let mut rows = vec![get_settings_button(
        renderer,
        "settings_para_validation_title.html",
        QueryType::NoOp,
    )?];
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorStartedParaValidating,
        "settings_item_started_para_validating.html",
        SettingsEditQueryType::StartedParaValidating,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorStoppedParaValidating,
        "settings_item_stopped_para_validating.html",
        SettingsEditQueryType::StoppedParaValidating,
        notification_rules,
    )?);
    rows.push(get_settings_button(
        renderer,
        "back.html",
        QueryType::SettingsNavigate(SettingsSubSection::ValidatorActivity),
    )?);
    rows.push(get_settings_button(
        renderer,
        "close.html",
        QueryType::Close,
    )?);
    Ok(InlineKeyboardMarkup {
        inline_keyboard: rows,
    })
}

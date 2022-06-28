use crate::messenger::button::settings::{get_notification_on_off_button, get_settings_button};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};
use tera::Tera;

pub(crate) fn get_active_inactive_settings_keyboard(
    renderer: &Tera,
    notification_rules: &[UserNotificationRule],
) -> anyhow::Result<InlineKeyboardMarkup> {
    let mut rows = vec![get_settings_button(
        renderer,
        "settings_active_inactive_title.html",
        QueryType::NoOp,
    )?];
    if let Some(item) = get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorActive,
        "settings_item_active.html",
        SettingsEditQueryType::Active,
        notification_rules,
    )? {
        rows.push(item);
    }
    if let Some(item) = get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorActiveNextSession,
        "settings_item_active_next_session.html",
        SettingsEditQueryType::ActiveNextSession,
        notification_rules,
    )? {
        rows.push(item);
    }
    if let Some(item) = get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorInactive,
        "settings_item_inactive.html",
        SettingsEditQueryType::Inactive,
        notification_rules,
    )? {
        rows.push(item);
    }
    if let Some(item) = get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorInactiveNextSession,
        "settings_item_inactive_next_session.html",
        SettingsEditQueryType::InactiveNextSession,
        notification_rules,
    )? {
        rows.push(item);
    }
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

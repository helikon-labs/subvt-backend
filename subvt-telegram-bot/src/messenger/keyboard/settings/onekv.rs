use crate::messenger::button::settings::{get_notification_on_off_button, get_settings_button};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};
use tera::Tera;

pub(crate) fn get_onekv_settings_keyboard(
    renderer: &Tera,
    notification_rules: &[UserNotificationRule],
) -> anyhow::Result<InlineKeyboardMarkup> {
    let mut rows = vec![get_settings_button(
        renderer,
        "settings_onekv_title.html",
        QueryType::NoOp,
    )?];
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::OneKVValidatorBinaryVersionChange,
        "settings_item_onekv_binary_version_change.html",
        SettingsEditQueryType::OneKVBinaryVersionChange,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::OneKVValidatorLocationChange,
        "settings_item_onekv_location_change.html",
        SettingsEditQueryType::OneKVLocationChange,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::OneKVValidatorOnlineStatusChange,
        "settings_item_onekv_online_status_change.html",
        SettingsEditQueryType::OneKVOnlineStatusChange,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::OneKVValidatorRankChange,
        "settings_item_onekv_rank_change.html",
        SettingsEditQueryType::OneKVRankChange,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::OneKVValidatorValidityChange,
        "settings_item_onekv_validity_change.html",
        SettingsEditQueryType::OneKVValidityChange,
        notification_rules,
    )?);
    rows.push(get_settings_button(
        renderer,
        "back.html",
        QueryType::SettingsNavigate(SettingsSubSection::Root),
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

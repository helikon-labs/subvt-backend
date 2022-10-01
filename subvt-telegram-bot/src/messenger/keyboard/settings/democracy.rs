use crate::messenger::button::settings::{get_notification_on_off_button, get_settings_button};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::notification::{NotificationTypeCode, UserNotificationRule};
use tera::Tera;

pub(crate) fn get_democracy_settings_keyboard(
    renderer: &Tera,
    notification_rules: &[UserNotificationRule],
) -> anyhow::Result<InlineKeyboardMarkup> {
    let mut rows = vec![get_settings_button(
        renderer,
        "settings_democracy_title.html",
        QueryType::NoOp,
    )?];
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyProposed,
        "settings_item_democracy_proposed.html",
        SettingsEditQueryType::DemocracyProposed,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracySeconded,
        "settings_item_democracy_seconded.html",
        SettingsEditQueryType::DemocracySeconded,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyStarted,
        "settings_item_democracy_started.html",
        SettingsEditQueryType::DemocracyStarted,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyCancelled,
        "settings_item_democracy_cancelled.html",
        SettingsEditQueryType::DemocracyCancelled,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyPassed,
        "settings_item_democracy_passed.html",
        SettingsEditQueryType::DemocracyPassed,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyNotPassed,
        "settings_item_democracy_not_passed.html",
        SettingsEditQueryType::DemocracyNotPassed,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyVoted,
        "settings_item_democracy_voted.html",
        SettingsEditQueryType::DemocracyVoted,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyDelegated,
        "settings_item_democracy_delegated.html",
        SettingsEditQueryType::DemocracyDelegated,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::DemocracyUndelegated,
        "settings_item_democracy_undelegated.html",
        SettingsEditQueryType::DemocracyUndelegated,
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

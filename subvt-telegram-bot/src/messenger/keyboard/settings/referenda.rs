use crate::messenger::button::settings::{get_notification_on_off_button, get_settings_button};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::notification::{NotificationTypeCode, UserNotificationRule};
use tera::Tera;

pub(crate) fn get_referenda_settings_keyboard(
    renderer: &Tera,
    notification_rules: &[UserNotificationRule],
) -> anyhow::Result<InlineKeyboardMarkup> {
    let mut rows = vec![get_settings_button(
        renderer,
        "settings_referenda_title.html",
        QueryType::NoOp,
    )?];
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumApproved,
        "settings_item_referendum_approved.html",
        SettingsEditQueryType::ReferendumApproved,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumCancelled,
        "settings_item_referendum_cancelled.html",
        SettingsEditQueryType::ReferendumCancelled,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumConfirmed,
        "settings_item_referendum_confirmed.html",
        SettingsEditQueryType::ReferendumConfirmed,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumDecisionStarted,
        "settings_item_referendum_decision_started.html",
        SettingsEditQueryType::ReferendumDecisionStarted,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumKilled,
        "settings_item_referendum_killed.html",
        SettingsEditQueryType::ReferendumKilled,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumRejected,
        "settings_item_referendum_rejected.html",
        SettingsEditQueryType::ReferendumRejected,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumSubmitted,
        "settings_item_referendum_submitted.html",
        SettingsEditQueryType::ReferendumSubmitted,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ReferendumTimedOut,
        "settings_item_referendum_timed_out.html",
        SettingsEditQueryType::ReferendumTimedOut,
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

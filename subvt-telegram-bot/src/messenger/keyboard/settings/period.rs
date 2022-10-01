use crate::messenger::button::settings::{get_notification_period_button, get_settings_button};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::notification::{
    NotificationPeriodType, NotificationTypeCode, UserNotificationRule,
};
use tera::Tera;

pub(crate) fn get_period_settings_keyboard(
    renderer: &Tera,
    edit_type: SettingsEditQueryType,
    notification_type_code: NotificationTypeCode,
    back_target: SettingsSubSection,
    notification_rules: &[UserNotificationRule],
) -> anyhow::Result<InlineKeyboardMarkup> {
    let mut rows = vec![get_settings_button(
        renderer,
        match edit_type {
            SettingsEditQueryType::BlockAuthorship => "settings_block_authorship_title.html",
            SettingsEditQueryType::NewNomination => "settings_new_nominations_title.html",
            SettingsEditQueryType::LostNomination => "settings_lost_nominations_title.html",
            _ => panic!(
                "Period settings keyboard not implemented for edit query type {:?}.",
                edit_type
            ),
        },
        QueryType::NoOp,
    )?];
    if let Some(button) = get_notification_period_button(
        renderer,
        notification_type_code,
        edit_type,
        NotificationPeriodType::Off,
        0,
        notification_rules,
    )? {
        rows.push(button);
    };
    if let Some(button) = get_notification_period_button(
        renderer,
        notification_type_code,
        edit_type,
        NotificationPeriodType::Immediate,
        0,
        notification_rules,
    )? {
        rows.push(button);
    };
    if let Some(button) = get_notification_period_button(
        renderer,
        notification_type_code,
        edit_type,
        NotificationPeriodType::Hour,
        1,
        notification_rules,
    )? {
        rows.push(button);
    };
    if let Some(button) = get_notification_period_button(
        renderer,
        notification_type_code,
        edit_type,
        NotificationPeriodType::Hour,
        2,
        notification_rules,
    )? {
        rows.push(button);
    };
    if let Some(button) = get_notification_period_button(
        renderer,
        notification_type_code,
        edit_type,
        NotificationPeriodType::Epoch,
        3,
        notification_rules,
    )? {
        rows.push(button);
    };
    if let Some(button) = get_notification_period_button(
        renderer,
        notification_type_code,
        edit_type,
        NotificationPeriodType::Era,
        1,
        notification_rules,
    )? {
        rows.push(button);
    };

    rows.push(get_settings_button(
        renderer,
        "back.html",
        QueryType::SettingsNavigate(back_target),
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

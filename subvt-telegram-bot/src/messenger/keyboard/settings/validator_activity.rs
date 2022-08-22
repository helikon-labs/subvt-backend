use crate::messenger::button::settings::{get_notification_on_off_button, get_settings_button};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};
use tera::Tera;

pub(crate) fn get_validator_activity_settings_keyboard(
    renderer: &Tera,
    notification_rules: &[UserNotificationRule],
) -> anyhow::Result<InlineKeyboardMarkup> {
    let mut rows = vec![
        get_settings_button(
            renderer,
            "settings_validator_activity_title.html",
            QueryType::NoOp,
        )?,
        get_settings_button(
            renderer,
            "settings_active_inactive.html",
            QueryType::SettingsNavigate(SettingsSubSection::ActiveInactive),
        )?,
        get_settings_button(
            renderer,
            "settings_item_block_authorship.html",
            QueryType::SettingsNavigate(SettingsSubSection::BlockAuthorship),
        )?,
        get_settings_button(
            renderer,
            "settings_item_para_validation.html",
            QueryType::SettingsNavigate(SettingsSubSection::ParaValidation),
        )?,
    ];
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorChilled,
        "settings_item_chilled.html",
        SettingsEditQueryType::Chilled,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorSetController,
        "settings_item_set_controller.html",
        SettingsEditQueryType::SetController,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorIdentityChanged,
        "settings_item_id_changed.html",
        SettingsEditQueryType::IdentityChanged,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorOfflineOffence,
        "settings_item_offline_offence.html",
        SettingsEditQueryType::OfflineOffence,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorPayoutStakers,
        "settings_item_payout_stakers.html",
        SettingsEditQueryType::PayoutStakers,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorSessionKeysChanged,
        "settings_item_session_keys_changed.html",
        SettingsEditQueryType::SessionKeysChanged,
        notification_rules,
    )?);
    rows.push(get_notification_on_off_button(
        renderer,
        NotificationTypeCode::ChainValidatorUnclaimedPayout,
        "settings_item_unclaimed_payout.html",
        SettingsEditQueryType::UnclaimedPayout,
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

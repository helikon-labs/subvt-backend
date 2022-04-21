use super::Messenger;
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};

impl Messenger {
    pub(crate) fn get_active_inactive_settings_keyboard(
        &self,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows =
            vec![self.get_settings_button("settings_active_inactive_title.html", QueryType::NoOp)?];
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorActive,
            "settings_item_active.html",
            SettingsEditQueryType::Active,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorActiveNextSession,
            "settings_item_active_next_session.html",
            SettingsEditQueryType::ActiveNextSession,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorInactive,
            "settings_item_inactive.html",
            SettingsEditQueryType::Inactive,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorInactiveNextSession,
            "settings_item_inactive_next_session.html",
            SettingsEditQueryType::InactiveNextSession,
            notification_rules,
        )? {
            rows.push(item);
        }
        rows.push(self.get_settings_button(
            "back.html",
            QueryType::SettingsNavigate(SettingsSubSection::ValidatorActivity),
        )?);
        rows.push(self.get_settings_button("close.html", QueryType::Close)?);
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }
}

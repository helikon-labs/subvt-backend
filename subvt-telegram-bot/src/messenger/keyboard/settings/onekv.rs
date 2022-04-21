use super::Messenger;
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};

impl Messenger {
    pub(crate) fn get_onekv_settings_keyboard(
        &self,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows =
            vec![self.get_settings_button("settings_onekv_title.html", QueryType::NoOp)?];
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::OneKVValidatorBinaryVersionChange,
            "settings_item_onekv_binary_version_change.html",
            SettingsEditQueryType::OneKVBinaryVersionChange,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::OneKVValidatorLocationChange,
            "settings_item_onekv_location_change.html",
            SettingsEditQueryType::OneKVLocationChange,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::OneKVValidatorRankChange,
            "settings_item_onekv_rank_change.html",
            SettingsEditQueryType::OneKVRankChange,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::OneKVValidatorValidityChange,
            "settings_item_onekv_validity_change.html",
            SettingsEditQueryType::OneKVValidityChange,
            notification_rules,
        )? {
            rows.push(item);
        }
        rows.push(self.get_settings_button(
            "back.html",
            QueryType::SettingsNavigate(SettingsSubSection::Root),
        )?);
        rows.push(self.get_settings_button("close.html", QueryType::Close)?);
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }
}

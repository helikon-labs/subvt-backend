use super::Messenger;
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationPeriodType, NotificationTypeCode, UserNotificationRule};

impl Messenger {
    pub(crate) fn get_period_settings_keyboard(
        &self,
        edit_type: SettingsEditQueryType,
        notification_type_code: NotificationTypeCode,
        back_target: SettingsSubSection,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows = vec![self.get_settings_button(
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
        if let Some(button) = self.get_notification_period_button(
            notification_type_code,
            edit_type,
            NotificationPeriodType::Off,
            0,
            notification_rules,
        )? {
            rows.push(button);
        };
        if let Some(button) = self.get_notification_period_button(
            notification_type_code,
            edit_type,
            NotificationPeriodType::Immediate,
            0,
            notification_rules,
        )? {
            rows.push(button);
        };
        if let Some(button) = self.get_notification_period_button(
            notification_type_code,
            edit_type,
            NotificationPeriodType::Hour,
            1,
            notification_rules,
        )? {
            rows.push(button);
        };
        if let Some(button) = self.get_notification_period_button(
            notification_type_code,
            edit_type,
            NotificationPeriodType::Hour,
            2,
            notification_rules,
        )? {
            rows.push(button);
        };
        if let Some(button) = self.get_notification_period_button(
            notification_type_code,
            edit_type,
            NotificationPeriodType::Epoch,
            3,
            notification_rules,
        )? {
            rows.push(button);
        };
        if let Some(button) = self.get_notification_period_button(
            notification_type_code,
            edit_type,
            NotificationPeriodType::Era,
            1,
            notification_rules,
        )? {
            rows.push(button);
        };

        rows.push(self.get_settings_button("back.html", QueryType::SettingsNavigate(back_target))?);
        rows.push(self.get_settings_button("close.html", QueryType::Close)?);
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }
}

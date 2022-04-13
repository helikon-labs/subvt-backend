use super::Messenger;
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};

impl Messenger {
    pub(crate) fn get_democracy_settings_keyboard(
        &self,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows =
            vec![self.get_settings_button("settings_democracy_title.html", QueryType::NoOp)?];
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyProposed,
            "settings_item_democracy_proposed.html",
            SettingsEditQueryType::DemocracyProposed,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracySeconded,
            "settings_item_democracy_seconded.html",
            SettingsEditQueryType::DemocracySeconded,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyStarted,
            "settings_item_democracy_started.html",
            SettingsEditQueryType::DemocracyStarted,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyCancelled,
            "settings_item_democracy_cancelled.html",
            SettingsEditQueryType::DemocracyCancelled,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyPassed,
            "settings_item_democracy_passed.html",
            SettingsEditQueryType::DemocracyPassed,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyNotPassed,
            "settings_item_democracy_not_passed.html",
            SettingsEditQueryType::DemocracyNotPassed,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyVoted,
            "settings_item_democracy_voted.html",
            SettingsEditQueryType::DemocracyVoted,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyDelegated,
            "settings_item_democracy_delegated.html",
            SettingsEditQueryType::DemocracyDelegated,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::DemocracyUndelegated,
            "settings_item_democracy_undelegated.html",
            SettingsEditQueryType::DemocracyUndelegated,
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

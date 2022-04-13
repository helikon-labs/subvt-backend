use super::Messenger;
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use frankenstein::InlineKeyboardMarkup;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};

impl Messenger {
    pub(crate) fn get_validator_activity_settings_keyboard(
        &self,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows = vec![
            self.get_settings_button("settings_validator_activity_title.html", QueryType::NoOp)?,
            self.get_settings_button(
                "settings_active_inactive.html",
                QueryType::SettingsNavigate(SettingsSubSection::ActiveInactive),
            )?,
            self.get_settings_button(
                "settings_item_block_authorship.html",
                QueryType::SettingsNavigate(SettingsSubSection::BlockAuthorship),
            )?,
        ];
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorChilled,
            "settings_item_chilled.html",
            SettingsEditQueryType::Chilled,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorSetController,
            "settings_item_set_controller.html",
            SettingsEditQueryType::SetController,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorIdentityChanged,
            "settings_item_id_changed.html",
            SettingsEditQueryType::IdentityChanged,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorOfflineOffence,
            "settings_item_offline_offence.html",
            SettingsEditQueryType::OfflineOffence,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorPayoutStakers,
            "settings_item_payout_stakers.html",
            SettingsEditQueryType::PayoutStakers,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorSessionKeysChanged,
            "settings_item_session_keys_changed.html",
            SettingsEditQueryType::SessionKeysChanged,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_on_off_button(
            NotificationTypeCode::ChainValidatorUnclaimedPayout,
            "settings_item_unclaimed_payout.html",
            SettingsEditQueryType::UnclaimedPayout,
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

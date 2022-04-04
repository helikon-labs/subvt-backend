use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use crate::{Messenger, Query};
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup};
use subvt_types::app::{NotificationPeriodType, NotificationTypeCode, UserNotificationRule};
use tera::Context;

impl Messenger {
    fn get_settings_button(
        &self,
        template_file_name: &str,
        query_type: QueryType,
    ) -> anyhow::Result<Vec<InlineKeyboardButton>> {
        Ok(vec![InlineKeyboardButton {
            text: self.renderer.render(template_file_name, &Context::new())?,
            url: None,
            login_url: None,
            callback_data: Some(serde_json::to_string(&Query {
                query_type,
                parameter: None,
            })?),
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            callback_game: None,
            pay: None,
        }])
    }

    pub fn get_settings_keyboard(&self) -> anyhow::Result<InlineKeyboardMarkup> {
        let rows = vec![
            self.get_settings_button(
                "settings_validator_activity.html",
                QueryType::SettingsValidatorActivity,
            )?,
            self.get_settings_button("settings_nominations.html", QueryType::SettingsNominations)?,
            self.get_settings_button("settings_democracy.html", QueryType::SettingsDemocracy)?,
            self.get_settings_button("settings_onekv.html", QueryType::SettingsOneKV)?,
            self.get_settings_button("cancel.html", QueryType::Cancel)?,
        ];
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }
}

impl Messenger {
    fn get_notification_rule_button(
        &self,
        notification_type_code: NotificationTypeCode,
        template_file_name: &str,
        edit_type: SettingsEditQueryType,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<Option<Vec<InlineKeyboardButton>>> {
        if let Some(active_rule) = notification_rules
            .iter()
            .find(|rule| rule.notification_type.code == notification_type_code.to_string())
        {
            let is_on = active_rule.period_type == NotificationPeriodType::Immediate;
            let mut context = Context::new();
            context.insert("is_on", &is_on);
            Ok(Some(vec![InlineKeyboardButton {
                text: self.renderer.render(template_file_name, &context)?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::SettingsEdit(edit_type),
                    parameter: Some(serde_json::to_string(&!is_on)?),
                })?),
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }]))
        } else {
            Ok(None)
        }
    }

    pub fn get_validator_activity_settings_keyboard(
        &self,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows = vec![];
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorChilled,
            "settings_item_chilled.html",
            SettingsEditQueryType::Chilled,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorSetController,
            "settings_item_set_controller.html",
            SettingsEditQueryType::SetController,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorIdentityChanged,
            "settings_item_id_changed.html",
            SettingsEditQueryType::IdentityChanged,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorActive,
            "settings_item_active.html",
            SettingsEditQueryType::Active,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorActiveNextSession,
            "settings_item_active_next_session.html",
            SettingsEditQueryType::ActiveNextSession,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorInactive,
            "settings_item_inactive.html",
            SettingsEditQueryType::Inactive,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorInactiveNextSession,
            "settings_item_inactive_next_session.html",
            SettingsEditQueryType::InactiveNextSession,
            notification_rules,
        )? {
            rows.push(item);
        }

        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorOfflineOffence,
            "settings_item_offline_offence.html",
            SettingsEditQueryType::OfflineOffence,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorPayoutStakers,
            "settings_item_payout_stakers.html",
            SettingsEditQueryType::PayoutStakers,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorSessionKeysChanged,
            "settings_item_session_keys_changed.html",
            SettingsEditQueryType::SessionKeysChanged,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::ChainValidatorUnclaimedPayout,
            "settings_item_unclaimed_payout.html",
            SettingsEditQueryType::UnclaimedPayout,
            notification_rules,
        )? {
            rows.push(item);
        }

        rows.push(self.get_settings_button(
            "back.html",
            QueryType::SettingsBack(SettingsSubSection::Root),
        )?);
        rows.push(self.get_settings_button("cancel.html", QueryType::Cancel)?);
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }

    pub fn get_democracy_settings_keyboard(
        &self,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows = vec![];
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyProposed,
            "settings_item_democracy_proposed.html",
            SettingsEditQueryType::DemocracyProposed,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracySeconded,
            "settings_item_democracy_seconded.html",
            SettingsEditQueryType::DemocracySeconded,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyStarted,
            "settings_item_democracy_started.html",
            SettingsEditQueryType::DemocracyStarted,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyCancelled,
            "settings_item_democracy_cancelled.html",
            SettingsEditQueryType::DemocracyCancelled,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyPassed,
            "settings_item_democracy_passed.html",
            SettingsEditQueryType::DemocracyPassed,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyNotPassed,
            "settings_item_democracy_not_passed.html",
            SettingsEditQueryType::DemocracyNotPassed,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyVoted,
            "settings_item_democracy_voted.html",
            SettingsEditQueryType::DemocracyVoted,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyDelegated,
            "settings_item_democracy_delegated.html",
            SettingsEditQueryType::DemocracyDelegated,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::DemocracyUndelegated,
            "settings_item_democracy_undelegated.html",
            SettingsEditQueryType::DemocracyUndelegated,
            notification_rules,
        )? {
            rows.push(item);
        }

        rows.push(self.get_settings_button(
            "back.html",
            QueryType::SettingsBack(SettingsSubSection::Root),
        )?);
        rows.push(self.get_settings_button("cancel.html", QueryType::Cancel)?);
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }

    pub fn get_onekv_settings_keyboard(
        &self,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<InlineKeyboardMarkup> {
        let mut rows = vec![];
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::OneKVValidatorRankChange,
            "settings_item_onekv_rank_change.html",
            SettingsEditQueryType::OneKVRankChange,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::OneKVValidatorBinaryVersionChange,
            "settings_item_onekv_binary_version_change.html",
            SettingsEditQueryType::OneKVBinaryVersionChange,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::OneKVValidatorValidityChange,
            "settings_item_onekv_validity_change.html",
            SettingsEditQueryType::OneKVValidityChange,
            notification_rules,
        )? {
            rows.push(item);
        }
        if let Some(item) = self.get_notification_rule_button(
            NotificationTypeCode::OneKVValidatorLocationChange,
            "settings_item_onekv_location_change.html",
            SettingsEditQueryType::OneKVLocationChange,
            notification_rules,
        )? {
            rows.push(item);
        }

        rows.push(self.get_settings_button(
            "back.html",
            QueryType::SettingsBack(SettingsSubSection::Root),
        )?);
        rows.push(self.get_settings_button("cancel.html", QueryType::Cancel)?);
        Ok(InlineKeyboardMarkup {
            inline_keyboard: rows,
        })
    }
}

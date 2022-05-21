//! Manages the creation of different types of `/settings` buttons.
use super::super::Messenger;
use crate::query::{Query, QueryType, SettingsEditQueryType};
use crate::CONFIG;
use frankenstein::InlineKeyboardButton;
use subvt_types::app::{NotificationPeriodType, NotificationTypeCode, UserNotificationRule};
use tera::Context;

impl Messenger {
    pub(crate) fn get_settings_button(
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
            web_app: None,
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            callback_game: None,
            pay: None,
        }])
    }

    pub(crate) fn get_notification_on_off_button(
        &self,
        notification_type_code: NotificationTypeCode,
        template_file_name: &str,
        edit_type: SettingsEditQueryType,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<Option<Vec<InlineKeyboardButton>>> {
        if let Some(rule) = notification_rules
            .iter()
            .find(|rule| rule.notification_type.code == notification_type_code.to_string())
        {
            let is_on = rule.period_type == NotificationPeriodType::Immediate;
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
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }]))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get_notification_period_button(
        &self,
        notification_type_code: NotificationTypeCode,
        edit_type: SettingsEditQueryType,
        period_type: NotificationPeriodType,
        period: u16,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<Option<Vec<InlineKeyboardButton>>> {
        if let Some(rule) = notification_rules
            .iter()
            .find(|rule| rule.notification_type.code == notification_type_code.to_string())
        {
            let is_selected = rule.period_type == period_type && rule.period == period;
            let mut context = Context::new();
            context.insert("is_selected", &is_selected);
            context.insert("period_type", &period_type.to_string());
            context.insert("period", &period);
            context.insert("epochs_per_era", &CONFIG.substrate.epochs_per_era);
            let parameter = (period_type, period);
            Ok(Some(vec![InlineKeyboardButton {
                text: self
                    .renderer
                    .render("settings_item_notification_period.html", &context)?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::SettingsEdit(edit_type),
                    parameter: Some(serde_json::to_string(&parameter)?),
                })?),
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }]))
        } else {
            Ok(None)
        }
    }
}

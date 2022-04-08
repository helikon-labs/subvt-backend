use crate::query::{Query, QueryType};
use crate::query::{SettingsEditQueryType, SettingsSubSection};
use crate::TelegramBotError;
use frankenstein::{
    AnswerCallbackQueryParams, AsyncApi, AsyncTelegramApi, ChatId, DeleteMessageParams,
    EditMessageResponse, EditMessageTextParams, Error, InlineKeyboardButton, InlineKeyboardMarkup,
    Message as TelegramMessage, MethodResponse, ReplyMarkup, SendMessageParams,
};
use message::MessageType;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};
use subvt_utility::text::get_condensed_address;
use tera::{Context, Tera};

pub mod message;
pub mod settings;

const FORBIDDEN_ERROR_CODE: u64 = 403;

pub struct Messenger {
    api: AsyncApi,
    renderer: Tera,
}

impl Messenger {
    pub fn new(config: &Config, api: AsyncApi) -> anyhow::Result<Messenger> {
        let renderer = Tera::new(&format!(
            "{}{}telegram{}dialog{}*.html",
            config.notification_processor.template_dir_path,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
        ))?;
        Ok(Messenger { api, renderer })
    }
}

impl Messenger {
    pub async fn answer_callback_query(
        &self,
        callback_query_id: &str,
        text: Option<String>,
    ) -> anyhow::Result<MethodResponse<bool>> {
        let params = AnswerCallbackQueryParams {
            callback_query_id: callback_query_id.to_string(),
            text,
            show_alert: None,
            url: None,
            cache_time: None,
        };
        match self.api.answer_callback_query(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }

    pub async fn delete_message(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> anyhow::Result<MethodResponse<bool>> {
        let params = DeleteMessageParams {
            chat_id: ChatId::Integer(chat_id),
            message_id,
        };
        match self.api.delete_message(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }

    pub async fn send_message(
        &self,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        chat_id: i64,
        message_type: Box<MessageType>,
    ) -> anyhow::Result<MethodResponse<TelegramMessage>> {
        let inline_keyboard = match &*message_type {
            MessageType::BroadcastConfirm => {
                let rows = vec![
                    vec![InlineKeyboardButton {
                        text: "Yes".to_string(),
                        url: None,
                        login_url: None,
                        callback_data: Some(serde_json::to_string(&Query {
                            query_type: QueryType::ConfirmBroadcast,
                            parameter: None,
                        })?),
                        switch_inline_query: None,
                        switch_inline_query_current_chat: None,
                        callback_game: None,
                        pay: None,
                    }],
                    vec![InlineKeyboardButton {
                        text: "No".to_string(),
                        url: None,
                        login_url: None,
                        callback_data: Some(serde_json::to_string(&Query {
                            query_type: QueryType::Cancel,
                            parameter: None,
                        })?),
                        switch_inline_query: None,
                        switch_inline_query_current_chat: None,
                        callback_game: None,
                        pay: None,
                    }],
                ];
                Some(ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
                    inline_keyboard: rows,
                }))
            }
            MessageType::ValidatorList {
                validators,
                query_type,
            } => {
                if validators.is_empty() {
                    None
                } else {
                    let mut rows = vec![];
                    for validator in validators {
                        let query = Query {
                            query_type: *query_type,
                            parameter: Some(validator.id.to_string()),
                        };
                        rows.push(vec![InlineKeyboardButton {
                            text: if let Some(display) = &validator.display {
                                display.to_owned()
                            } else {
                                get_condensed_address(&validator.address, None)
                            },
                            url: None,
                            login_url: None,
                            callback_data: Some(serde_json::to_string(&query)?),
                            switch_inline_query: None,
                            switch_inline_query_current_chat: None,
                            callback_game: None,
                            pay: None,
                        }]);
                    }
                    rows.push(vec![InlineKeyboardButton {
                        text: self.renderer.render("cancel.html", &Context::new())?,
                        url: None,
                        login_url: None,
                        callback_data: Some(serde_json::to_string(&Query {
                            query_type: QueryType::Cancel,
                            parameter: None,
                        })?),
                        switch_inline_query: None,
                        switch_inline_query_current_chat: None,
                        callback_game: None,
                        pay: None,
                    }]);
                    Some(ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
                        inline_keyboard: rows,
                    }))
                }
            }
            MessageType::NominationSummary {
                chat_validator_id,
                validator_details,
            } => {
                if validator_details.nominations.is_empty() {
                    None
                } else {
                    let query = Query {
                        query_type: QueryType::NominationDetails,
                        parameter: Some(chat_validator_id.to_string()),
                    };
                    let rows = vec![vec![InlineKeyboardButton {
                        text: self
                            .renderer
                            .render("view_nomination_details.html", &Context::new())?,
                        url: None,
                        login_url: None,
                        callback_data: Some(serde_json::to_string(&query)?),
                        switch_inline_query: None,
                        switch_inline_query_current_chat: None,
                        callback_game: None,
                        pay: None,
                    }]];
                    Some(ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
                        inline_keyboard: rows,
                    }))
                }
            }
            MessageType::Settings => Some(ReplyMarkup::InlineKeyboardMarkup(
                self.get_settings_keyboard()?,
            )),
            _ => None,
        };
        let params = SendMessageParams {
            chat_id: ChatId::Integer(chat_id),
            text: message_type.get_content(&self.renderer),
            parse_mode: Some(frankenstein::ParseMode::Html),
            entities: None,
            disable_web_page_preview: Some(true),
            disable_notification: None,
            protect_content: None,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            reply_markup: inline_keyboard,
        };
        log::info!(
            "Message to chat {}: {}",
            chat_id,
            params.text.replace('\n', ""),
        );
        match self.api.send_message(&params).await {
            Ok(response) => Ok(response),
            Err(error) => {
                if let Error::ApiError(ref api_error) = error {
                    if api_error.error_code == FORBIDDEN_ERROR_CODE {
                        // chat blocked, delete app user and chat
                        let app_user_id = network_postgres.get_chat_app_user_id(chat_id).await?;
                        app_postgres.delete_user(app_user_id).await?;
                        network_postgres.delete_chat(chat_id).await?;
                    }
                }
                Err(TelegramBotError::Error(format!("{:?}", error)).into())
            }
        }
    }

    pub async fn update_settings_message(
        &self,
        chat_id: i64,
        settings_message_id: i32,
        sub_section: SettingsSubSection,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<EditMessageResponse> {
        let inline_keyboard = match sub_section {
            SettingsSubSection::Root => self.get_settings_keyboard()?,
            SettingsSubSection::ValidatorActivity => {
                self.get_validator_activity_settings_keyboard(notification_rules)?
            }
            SettingsSubSection::ActiveInactive => {
                self.get_active_inactive_settings_keyboard(notification_rules)?
            }
            SettingsSubSection::BlockAuthorship => self.get_period_settings_keyboard(
                SettingsEditQueryType::BlockAuthorship,
                NotificationTypeCode::ChainValidatorBlockAuthorship,
                SettingsSubSection::ValidatorActivity,
                notification_rules,
            )?,
            SettingsSubSection::Democracy => {
                self.get_democracy_settings_keyboard(notification_rules)?
            }
            SettingsSubSection::OneKV => self.get_onekv_settings_keyboard(notification_rules)?,
            SettingsSubSection::Nominations => self.get_nomination_settings_keyboard()?,
            SettingsSubSection::NewNomination => self.get_period_settings_keyboard(
                SettingsEditQueryType::NewNomination,
                NotificationTypeCode::ChainValidatorNewNomination,
                SettingsSubSection::Nominations,
                notification_rules,
            )?,
            SettingsSubSection::LostNomination => self.get_period_settings_keyboard(
                SettingsEditQueryType::LostNomination,
                NotificationTypeCode::ChainValidatorLostNomination,
                SettingsSubSection::Nominations,
                notification_rules,
            )?,
        };
        let params = EditMessageTextParams {
            chat_id: Some(ChatId::Integer(chat_id)),
            message_id: Some(settings_message_id),
            inline_message_id: None,
            text: self
                .renderer
                .render("settings_prompt.html", &Context::new())?,
            parse_mode: Some(frankenstein::ParseMode::Html),
            entities: None,
            disable_web_page_preview: Some(true),
            reply_markup: Some(inline_keyboard),
        };
        match self.api.edit_message_text(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }
}

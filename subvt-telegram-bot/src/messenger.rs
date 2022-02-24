use crate::query::{Query, QueryType};
use crate::{TelegramBotError, CONFIG};
use frankenstein::{
    AnswerCallbackQueryParams, AsyncApi, AsyncTelegramApi, ChatId, DeleteMessageParams,
    InlineKeyboardButton, InlineKeyboardMarkup, Message, MethodResponse, ReplyMarkup,
    SendMessageParams,
};
use subvt_config::Config;
use subvt_types::subvt::ValidatorDetails;
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::{get_condensed_address, get_condensed_session_keys};
use tera::{Context, Tera};

pub enum MessageType {
    Intro,
    BadRequest,
    UnknownCommand(String),
    InvalidAddress(String),
    InvalidAddressTryAgain(String),
    ValidatorNotFound(String),
    ValidatorExistsOnChat(String),
    ValidatorAdded,
    AddValidator,
    ValidatorInfo(Box<ValidatorDetails>),
}

impl MessageType {
    pub fn get_content(&self, renderer: &Tera) -> String {
        let mut context = Context::new();
        let template_name = match self {
            Self::Intro => "introduction.html",
            Self::BadRequest => "bad_request.html",
            Self::UnknownCommand(command) => {
                context.insert("command", command);
                "unknown_command.html"
            }
            Self::InvalidAddress(address) => {
                context.insert("address", address);
                "invalid_address.html"
            }
            Self::InvalidAddressTryAgain(address) => {
                context.insert("address", address);
                "invalid_address_try_again.html"
            }
            Self::ValidatorNotFound(address) => {
                context.insert("condensed_address", &get_condensed_address(address));
                "validator_not_found.html"
            }
            Self::ValidatorExistsOnChat(address) => {
                context.insert("condensed_address", &get_condensed_address(address));
                "validator_exists_on_chat.html"
            }
            Self::ValidatorAdded => "validator_added.html",
            Self::AddValidator => "add_validator.html",
            Self::ValidatorInfo(validator_details) => {
                if let Some(display) = validator_details.get_full_display() {
                    context.insert("has_display", &true);
                    context.insert("display", &display);
                } else {
                    context.insert("has_display", &false);
                }
                context.insert("network", &CONFIG.substrate.chain);
                let address = validator_details.account.id.to_ss58_check();
                context.insert("address", &address);
                context.insert("condensed_address", &get_condensed_address(&address));
                let controller_address = validator_details.controller_account_id.to_ss58_check();
                context.insert("controller_address", &controller_address);
                context.insert(
                    "condensed_controller_address",
                    &get_condensed_address(&controller_address),
                );
                context.insert(
                    "condensed_session_keys",
                    &get_condensed_session_keys(&validator_details.next_session_keys)
                        .to_lowercase(),
                );
                context.insert("is_active", &validator_details.is_active);
                context.insert("is_para_validator", &validator_details.is_para_validator);
                context.insert(
                    "is_active_next_session",
                    &validator_details.active_next_session,
                );
                context.insert(
                    "commission",
                    &format_decimal(
                        validator_details.preferences.commission_per_billion as u128,
                        7,
                        2,
                        "%",
                    ),
                );
                context.insert(
                    "blocks_nominations",
                    &validator_details.preferences.blocks_nominations,
                );
                context.insert("oversubscribed", &validator_details.oversubscribed);
                if let Some(heartbeat_received) = validator_details.heartbeat_received {
                    context.insert("heartbeat_received", &heartbeat_received);
                }
                context.insert("slash_count", &validator_details.slash_count);
                "validator_info.html"
            }
        };
        renderer.render(template_name, &context).unwrap()
    }
}

pub struct Messenger {
    api: AsyncApi,
    renderer: Tera,
}

impl Messenger {
    pub fn new(config: &Config, api: AsyncApi) -> anyhow::Result<Messenger> {
        let renderer = Tera::new(&format!(
            "{}{}telegram{}dialog{}*.html",
            config.notification_sender.template_dir_path,
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
        chat_id: i64,
        message_type: MessageType,
    ) -> anyhow::Result<MethodResponse<Message>> {
        let params = SendMessageParams {
            chat_id: ChatId::Integer(chat_id),
            text: message_type.get_content(&self.renderer),
            parse_mode: Some("html".to_string()),
            entities: None,
            disable_web_page_preview: Some(true),
            disable_notification: None,
            protect_content: None,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            reply_markup: None,
        };
        match self.api.send_message(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }

    pub async fn send_validator_list(
        &self,
        chat_id: i64,
        query_type: QueryType,
        validators: &Vec<ValidatorDetails>,
    ) -> anyhow::Result<MethodResponse<Message>> {
        let mut rows = vec![];
        for validator in validators {
            let address = validator.account.id.to_ss58_check();
            let query = Query {
                query_type: query_type.clone(),
                parameter: Some(address.clone()),
            };
            rows.push(vec![InlineKeyboardButton {
                text: if let Some(display) = validator.get_full_display() {
                    display
                } else {
                    get_condensed_address(&address)
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
        let params = SendMessageParams {
            chat_id: ChatId::Integer(chat_id),
            text: self
                .renderer
                .render("select_validator.html", &Context::new())
                .unwrap(),
            parse_mode: Some("html".to_string()),
            entities: None,
            disable_web_page_preview: Some(true),
            disable_notification: None,
            protect_content: None,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            reply_markup: Some(ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
                inline_keyboard: rows,
            })),
        };
        match self.api.send_message(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }
}

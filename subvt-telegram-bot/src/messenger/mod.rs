//! This module handles the sending of all the messages to a Telegram chat.
use crate::api::{AsyncApi, Error};
use crate::query::{SettingsEditQueryType, SettingsSubSection};
use crate::{TelegramBotError, CONFIG};
use frankenstein::{
    AnswerCallbackQueryParams, AsyncTelegramApi, ChatId, DeleteMessageParams, EditMessageResponse,
    EditMessageTextParams, Message as TelegramMessage, MethodResponse, ParseMode, ReplyMarkup,
    SendMessageParams, SendPhotoParams,
};
use message::MessageType;
use std::path::Path;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::{NotificationTypeCode, UserNotificationRule};
use subvt_types::sub_id::NFTCollection;
use tera::{Context, Tera};

pub mod button;
pub mod keyboard;
pub mod message;

const FORBIDDEN_ERROR_CODE: u64 = 403;

/// Telegram messenger.
pub struct Messenger {
    /// Async Telegram API.
    api: AsyncApi,
    /// Template renderer.
    renderer: Tera,
}

impl Messenger {
    pub fn new() -> anyhow::Result<Messenger> {
        // init the renderer with the template collection
        let renderer = Tera::new(&format!(
            "{}{}telegram{}dialog{}*.html",
            CONFIG.notification_processor.template_dir_path,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
        ))?;
        // init the async Telegram API
        let api = AsyncApi::new(&CONFIG.telegram_bot.api_token);
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

    pub async fn send_image(
        &self,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        chat_id: i64,
        path: &Path,
    ) -> anyhow::Result<MethodResponse<TelegramMessage>> {
        let params = SendPhotoParams {
            chat_id: ChatId::Integer(chat_id),
            photo: frankenstein::api_params::File::InputFile(frankenstein::InputFile {
                path: path.into(),
            }),
            caption: None,
            parse_mode: Some(ParseMode::Html),
            caption_entities: None,
            disable_notification: None,
            protect_content: None,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            reply_markup: None,
        };
        match self.api.send_photo(&params).await {
            Ok(response) => Ok(response),
            Err(error) => {
                if let Error::Api(ref api_error) = error {
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

    /// Send a message with content indicated by the `message_type` parameter.
    pub async fn send_message(
        &self,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        chat_id: i64,
        message_type: Box<MessageType>,
    ) -> anyhow::Result<MethodResponse<TelegramMessage>> {
        // some messages need an inline keyboard, such as selection from the validator list,
        // selection from the NFT list, etc.
        let inline_keyboard = match &*message_type {
            MessageType::BroadcastConfirm => self.get_broadcast_confirm_keyboard()?,
            MessageType::ValidatorList {
                validators,
                query_type,
            } => self.get_validator_list_keyboard(validators, query_type)?,
            MessageType::NominationSummary {
                chat_validator_id,
                validator_details,
            } => self.get_nomination_summary_keyboard(*chat_validator_id, validator_details)?,
            MessageType::Settings => Some(ReplyMarkup::InlineKeyboardMarkup(
                self.get_settings_keyboard()?,
            )),
            MessageType::RefererendumList(posts) => self.get_referendum_list_keyboard(posts)?,
            MessageType::SelectContactType => self.get_contact_type_keyboard()?,
            MessageType::NFTs {
                validator_id,
                collection_page,
                page_index,
                has_prev,
                has_next,
                ..
            } => Some(ReplyMarkup::InlineKeyboardMarkup(
                self.get_nft_collection_keyboard(
                    *validator_id,
                    collection_page,
                    *page_index,
                    *has_prev,
                    *has_next,
                )?,
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
                if let Error::Api(ref api_error) = error {
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

    /// Inline keyboard displayed for the `/settings` command doesn't get recreated after every
    /// option change, but it gets updated after every change.
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

    /// `/nfts` command produces a paged list. This function manages the paging.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_nfts_message(
        &self,
        chat_id: i64,
        message_id: i32,
        validator_id: u64,
        total_count: usize,
        collection_page: NFTCollection,
        page_index: usize,
        has_prev: bool,
        has_next: bool,
    ) -> anyhow::Result<EditMessageResponse> {
        let params = EditMessageTextParams {
            chat_id: Some(ChatId::Integer(chat_id)),
            message_id: Some(message_id),
            inline_message_id: None,
            text: {
                let mut context = Context::new();
                context.insert("total_count", &total_count);
                self.renderer.render("select_nft.html", &context)?
            },
            parse_mode: Some(frankenstein::ParseMode::Html),
            entities: None,
            disable_web_page_preview: Some(true),
            reply_markup: Some(self.get_nft_collection_keyboard(
                validator_id,
                &collection_page,
                page_index,
                has_prev,
                has_next,
            )?),
        };
        match self.api.edit_message_text(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }
}

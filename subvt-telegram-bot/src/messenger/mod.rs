//! This module handles the sending of all the messages to a Telegram chat.
use crate::messenger::keyboard::referendum_tracks::get_referendum_tracks_keyboard;
use crate::messenger::keyboard::settings::referenda::get_referenda_settings_keyboard;
use crate::messenger::keyboard::{
    confirmation::get_confirmation_keyboard,
    contact_type::get_contact_type_keyboard,
    nft::get_nft_collection_keyboard,
    nomination_summary::get_nomination_summary_keyboard,
    referendum_list::get_referendum_list_keyboard,
    settings::{
        active_inactive::get_active_inactive_settings_keyboard, get_settings_keyboard,
        nomination::get_nomination_settings_keyboard, onekv::get_onekv_settings_keyboard,
        para_validation::get_para_validation_settings_keyboard,
        period::get_period_settings_keyboard,
        validator_activity::get_validator_activity_settings_keyboard,
    },
    validator_list::get_validator_list_keyboard,
};
use crate::query::{QueryType, SettingsEditQueryType, SettingsSubSection};
use crate::{TelegramBotError, CONFIG};
use async_trait::async_trait;
use frankenstein::{
    AnswerCallbackQueryParams, AsyncApi, AsyncTelegramApi, ChatId, DeleteMessageParams,
    EditMessageResponse, EditMessageTextParams, Error, Message as TelegramMessage, MethodResponse,
    ParseMode, ReplyMarkup, SendMessageParams, SendPhotoParams,
};
use message::MessageType;
#[cfg(test)]
use mockall::automock;
use std::path::Path;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::notification::{NotificationTypeCode, UserNotificationRule};
use subvt_types::sub_id::NFTCollection;
use tera::{Context, Tera};

pub mod button;
pub mod keyboard;
pub mod message;

const FORBIDDEN_ERROR_CODE: u64 = 403;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Messenger {
    async fn answer_callback_query(
        &self,
        callback_query_id: &str,
        text: Option<String>,
    ) -> anyhow::Result<MethodResponse<bool>>;

    async fn delete_message(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> anyhow::Result<MethodResponse<bool>>;

    async fn send_image(
        &self,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        chat_id: i64,
        path: &Path,
    ) -> anyhow::Result<MethodResponse<TelegramMessage>>;

    async fn send_message(
        &self,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        chat_id: i64,
        message_type: Box<MessageType>,
    ) -> anyhow::Result<MethodResponse<TelegramMessage>>;

    async fn update_settings_message(
        &self,
        chat_id: i64,
        settings_message_id: i32,
        sub_section: SettingsSubSection,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<EditMessageResponse>;

    #[allow(clippy::too_many_arguments)]
    async fn update_nfts_message(
        &self,
        chat_id: i64,
        message_id: i32,
        validator_id: u64,
        total_count: usize,
        collection_page: NFTCollection,
        page_index: usize,
        has_prev: bool,
        has_next: bool,
    ) -> anyhow::Result<EditMessageResponse>;
}

/// Telegram messenger.
pub struct MessengerImpl {
    /// Async Telegram API.
    api: AsyncApi,
    /// Template renderer.
    renderer: Tera,
}

impl MessengerImpl {
    pub fn new() -> anyhow::Result<MessengerImpl> {
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
        Ok(MessengerImpl { api, renderer })
    }
}

#[async_trait]
impl Messenger for MessengerImpl {
    async fn answer_callback_query(
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
            Err(error) => Err(TelegramBotError::Error(format!("{error:?}")).into()),
        }
    }

    async fn delete_message(
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
            Err(error) => Err(TelegramBotError::Error(format!("{error:?}")).into()),
        }
    }

    async fn send_image(
        &self,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        chat_id: i64,
        path: &Path,
    ) -> anyhow::Result<MethodResponse<TelegramMessage>> {
        let params = SendPhotoParams {
            chat_id: ChatId::Integer(chat_id),
            photo: frankenstein::api_params::FileUpload::InputFile(frankenstein::InputFile {
                path: path.into(),
            }),
            caption: None,
            parse_mode: Some(ParseMode::Html),
            caption_entities: None,
            has_spoiler: None,
            disable_notification: None,
            protect_content: None,
            reply_parameters: None,
            reply_markup: None,
            message_thread_id: None,
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
                Err(TelegramBotError::Error(format!("{error:?}")).into())
            }
        }
    }

    /// Send a message with content indicated by the `message_type` parameter.
    async fn send_message(
        &self,
        app_postgres: &PostgreSQLAppStorage,
        network_postgres: &PostgreSQLNetworkStorage,
        chat_id: i64,
        message_type: Box<MessageType>,
    ) -> anyhow::Result<MethodResponse<TelegramMessage>> {
        // some messages need an inline keyboard, such as selection from the validator list,
        // selection from the NFT list, etc.
        let inline_keyboard = match &*message_type {
            MessageType::BroadcastConfirm => {
                get_confirmation_keyboard(QueryType::ConfirmBroadcast)?
            }
            MessageType::RemoveAllValidatorsConfirm => {
                get_confirmation_keyboard(QueryType::RemoveAllValidators)?
            }
            MessageType::ValidatorList {
                validators,
                query_type,
            } => get_validator_list_keyboard(&self.renderer, validators, query_type)?,
            MessageType::NominationSummary {
                chat_validator_id,
                validator_details,
            } => get_nomination_summary_keyboard(
                &self.renderer,
                *chat_validator_id,
                validator_details,
            )?,
            MessageType::Settings => Some(ReplyMarkup::InlineKeyboardMarkup(
                get_settings_keyboard(&self.renderer)?,
            )),
            MessageType::ReferendumList(track_id, posts) => {
                get_referendum_list_keyboard(&self.renderer, *track_id, posts)?
            }
            MessageType::ReferendumTracks(data) => {
                get_referendum_tracks_keyboard(&self.renderer, data)?
            }
            MessageType::SelectContactType => get_contact_type_keyboard(&self.renderer)?,
            MessageType::NFTs {
                validator_id,
                collection_page,
                page_index,
                has_prev,
                has_next,
                ..
            } => Some(ReplyMarkup::InlineKeyboardMarkup(
                get_nft_collection_keyboard(
                    &self.renderer,
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
            parse_mode: Some(ParseMode::Html),
            entities: None,
            link_preview_options: None,
            disable_notification: None,
            protect_content: None,
            reply_parameters: None,
            reply_markup: inline_keyboard,
            message_thread_id: None,
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
                Err(TelegramBotError::Error(format!("{error:?}")).into())
            }
        }
    }

    /// Inline keyboard displayed for the `/settings` command doesn't get recreated after every
    /// option change, but it gets updated after every change.
    async fn update_settings_message(
        &self,
        chat_id: i64,
        settings_message_id: i32,
        sub_section: SettingsSubSection,
        notification_rules: &[UserNotificationRule],
    ) -> anyhow::Result<EditMessageResponse> {
        let inline_keyboard = match sub_section {
            SettingsSubSection::Root => get_settings_keyboard(&self.renderer)?,
            SettingsSubSection::ValidatorActivity => {
                get_validator_activity_settings_keyboard(&self.renderer, notification_rules)?
            }
            SettingsSubSection::ActiveInactive => {
                get_active_inactive_settings_keyboard(&self.renderer, notification_rules)?
            }
            SettingsSubSection::BlockAuthorship => get_period_settings_keyboard(
                &self.renderer,
                SettingsEditQueryType::BlockAuthorship,
                NotificationTypeCode::ChainValidatorBlockAuthorship,
                SettingsSubSection::ValidatorActivity,
                notification_rules,
            )?,
            SettingsSubSection::ParaValidation => {
                get_para_validation_settings_keyboard(&self.renderer, notification_rules)?
            }
            SettingsSubSection::OneKV => {
                get_onekv_settings_keyboard(&self.renderer, notification_rules)?
            }
            SettingsSubSection::Nominations => get_nomination_settings_keyboard(&self.renderer)?,
            SettingsSubSection::NewNomination => get_period_settings_keyboard(
                &self.renderer,
                SettingsEditQueryType::NewNomination,
                NotificationTypeCode::ChainValidatorNewNomination,
                SettingsSubSection::Nominations,
                notification_rules,
            )?,
            SettingsSubSection::LostNomination => get_period_settings_keyboard(
                &self.renderer,
                SettingsEditQueryType::LostNomination,
                NotificationTypeCode::ChainValidatorLostNomination,
                SettingsSubSection::Nominations,
                notification_rules,
            )?,
            SettingsSubSection::Referenda => {
                get_referenda_settings_keyboard(&self.renderer, notification_rules)?
            }
        };
        let params = EditMessageTextParams {
            chat_id: Some(ChatId::Integer(chat_id)),
            message_id: Some(settings_message_id),
            inline_message_id: None,
            text: self
                .renderer
                .render("settings_prompt.html", &Context::new())?,
            parse_mode: Some(ParseMode::Html),
            entities: None,
            link_preview_options: None,
            reply_markup: Some(inline_keyboard),
        };
        match self.api.edit_message_text(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{error:?}")).into()),
        }
    }

    /// `/nfts` command produces a paged list. This function manages the paging.
    #[allow(clippy::too_many_arguments)]
    async fn update_nfts_message(
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
            parse_mode: Some(ParseMode::Html),
            entities: None,
            link_preview_options: None,
            reply_markup: Some(get_nft_collection_keyboard(
                &self.renderer,
                validator_id,
                &collection_page,
                page_index,
                has_prev,
                has_next,
            )?),
        };
        match self.api.edit_message_text(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{error:?}")).into()),
        }
    }
}

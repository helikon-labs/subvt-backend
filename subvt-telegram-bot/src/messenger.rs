use crate::query::{Query, QueryType};
use crate::{TelegramBotError, CONFIG};
use chrono::{TimeZone, Utc};
use frankenstein::{
    AnswerCallbackQueryParams, AsyncApi, AsyncTelegramApi, ChatId, DeleteMessageParams,
    InlineKeyboardButton, InlineKeyboardMarkup, Message, MethodResponse, ReplyMarkup,
    SendMessageParams,
};
use itertools::Itertools;
use subvt_config::Config;
use subvt_types::crypto::AccountId;
use subvt_types::onekv::OneKVCandidateSummary;
use subvt_types::substrate::Balance;
use subvt_types::subvt::ValidatorDetails;
use subvt_types::telegram::TelegramChatValidator;
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::{get_condensed_address, get_condensed_session_keys};
use tera::{Context, Tera};

pub enum MessageType {
    Intro,
    Ok,
    BadRequest,
    GenericError,
    Broadcast,
    BroadcastConfirm,
    UnknownCommand(String),
    InvalidAddress(String),
    InvalidAddressTryAgain(String),
    ValidatorNotFound {
        maybe_address: Option<String>,
    },
    AddValidatorNotFound(String),
    ValidatorExistsOnChat(String),
    TooManyValidatorsOnChat,
    NoValidatorsOnChat,
    ValidatorAdded,
    AddValidator,
    ValidatorList {
        validators: Vec<TelegramChatValidator>,
        query_type: QueryType,
    },
    ValidatorInfo {
        address: String,
        maybe_validator_details: Box<Option<ValidatorDetails>>,
        maybe_onekv_candidate_summary: Box<Option<OneKVCandidateSummary>>,
    },
    NominationSummary {
        chat_validator_id: u64,
        validator_details: ValidatorDetails,
    },
    NominationDetails {
        validator_details: ValidatorDetails,
        onekv_nominator_account_ids: Vec<AccountId>,
    },
    ValidatorRemoved(TelegramChatValidator),
    Settings,
}

impl MessageType {
    pub fn get_content(&self, renderer: &Tera) -> String {
        let mut context = Context::new();
        let template_name = match self {
            Self::Intro => "introduction.html",
            Self::Ok => "ok.html",
            Self::BadRequest => "bad_request.html",
            Self::GenericError => "generic_error.html",
            Self::Broadcast => "broadcast.html",
            Self::BroadcastConfirm => "broadcast_confirm.html",
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
            Self::ValidatorNotFound { maybe_address } => {
                if let Some(address) = maybe_address {
                    context.insert("condensed_address", &get_condensed_address(address, None));
                }
                "validator_not_found.html"
            }
            Self::AddValidatorNotFound(address) => {
                context.insert("condensed_address", &get_condensed_address(address, None));
                "add_validator_not_found.html"
            }
            Self::ValidatorExistsOnChat(validator_display) => {
                context.insert("validator_display", validator_display);
                "validator_exists_on_chat.html"
            }
            Self::TooManyValidatorsOnChat => {
                context.insert(
                    "max_validators_per_chat",
                    &CONFIG.telegram_bot.max_validators_per_chat,
                );
                "too_many_validators_on_chat.html"
            }
            Self::NoValidatorsOnChat => "no_validators_on_chat.html",
            Self::ValidatorAdded => "validator_added.html",
            Self::AddValidator => "add_validator.html",
            Self::ValidatorList { .. } => "select_validator.html",
            Self::ValidatorInfo {
                address,
                maybe_validator_details,
                maybe_onekv_candidate_summary,
            } => {
                context.insert("condensed_address", &get_condensed_address(address, None));
                context.insert("is_validator", &maybe_validator_details.is_some());
                if let Some(validator_details) = &**maybe_validator_details {
                    if let Some(display) = validator_details.account.get_full_display() {
                        context.insert("has_display", &true);
                        context.insert("display", &display);
                    } else {
                        context.insert("has_display", &false);
                    }
                    context.insert("network", &CONFIG.substrate.chain);
                    let address = &validator_details.account.address;
                    context.insert("address", address);
                    context.insert("condensed_address", &get_condensed_address(address, None));
                    let controller_address =
                        validator_details.controller_account_id.to_ss58_check();
                    context.insert("controller_address", &controller_address);
                    context.insert(
                        "condensed_controller_address",
                        &get_condensed_address(&controller_address, None),
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
                }
                context.insert("is_onekv", &maybe_onekv_candidate_summary.is_some());
                if let Some(onekv_summary) = &**maybe_onekv_candidate_summary {
                    context.insert("onekv_name", &onekv_summary.name);
                    if let Some(location) = &onekv_summary.location {
                        context.insert("onekv_location", location);
                    }
                    let date_time_format = "%b %d, %Y %H:%M UTC";
                    let discovered_at =
                        Utc::timestamp(&Utc, onekv_summary.discovered_at as i64 / 1000, 0);
                    context.insert(
                        "onekv_discovered_at",
                        &discovered_at.format(date_time_format).to_string(),
                    );
                    if let Some(version) = &onekv_summary.version {
                        context.insert("onekv_version", version);
                    }
                    if let Some(nominated_at) = onekv_summary.nominated_at {
                        let nominated_at = Utc::timestamp(&Utc, nominated_at as i64 / 1000, 0);
                        context.insert(
                            "onekv_nominated_at",
                            &nominated_at.format(date_time_format).to_string(),
                        );
                    }
                    if onekv_summary.online_since > 0 {
                        let online_since =
                            Utc::timestamp(&Utc, onekv_summary.online_since as i64 / 1000, 0);
                        context.insert(
                            "onekv_online_since",
                            &online_since.format(date_time_format).to_string(),
                        );
                    } else if onekv_summary.offline_since > 0 {
                        let offline_since =
                            Utc::timestamp(&Utc, onekv_summary.offline_since as i64 / 1000, 0);
                        context.insert(
                            "onekv_offline_since",
                            &offline_since.format(date_time_format).to_string(),
                        );
                    }
                    if let Some(rank) = onekv_summary.rank {
                        context.insert("onekv_rank", &rank);
                    }
                    if let Some(score) = onekv_summary.total_score {
                        context.insert("onekv_score", &(score as u64));
                    }
                    let is_valid = onekv_summary.is_valid();
                    context.insert("onekv_is_valid", &is_valid);
                    if !is_valid {
                        let invalidity_reasons: Vec<String> = onekv_summary
                            .validity
                            .iter()
                            .filter(|v| !v.is_valid)
                            .map(|v| v.details.clone())
                            .collect();
                        context.insert("onekv_invalidity_reasons", &invalidity_reasons);
                    }
                    context.insert(
                        "onekv_democracy_vote_count",
                        &onekv_summary.democracy_vote_count,
                    );
                    context.insert(
                        "onekv_council_vote_count",
                        &onekv_summary.council_votes.len(),
                    );
                    let last_updated =
                        Utc::timestamp(&Utc, onekv_summary.record_created_at as i64 / 1000, 0);
                    context.insert(
                        "onekv_last_updated",
                        &last_updated.format(date_time_format).to_string(),
                    );
                }
                "validator_info.html"
            }
            Self::NominationSummary {
                validator_details, ..
            } => {
                let self_stake = validator_details.self_stake.total_amount;
                let (
                    active_nominator_count,
                    active_nomination_total,
                    inactive_nominator_count,
                    inactive_nomination_total,
                ) = if let Some(validator_stake) = &validator_details.validator_stake {
                    let active_nominator_account_ids: Vec<AccountId> = validator_stake
                        .nominators
                        .iter()
                        .map(|n| n.account.id)
                        .collect();
                    let mut inactive_nominator_count: usize = 0;
                    let mut inactive_nomination_total: Balance = 0;
                    for nomination in &validator_details.nominations {
                        if !active_nominator_account_ids.contains(&nomination.stash_account.id) {
                            inactive_nominator_count += 1;
                            inactive_nomination_total += nomination.stake.active_amount;
                        }
                    }
                    (
                        active_nominator_account_ids.len(),
                        validator_stake.total_stake,
                        inactive_nominator_count,
                        inactive_nomination_total,
                    )
                } else {
                    let inactive_nomination_total: Balance = validator_details
                        .nominations
                        .iter()
                        .map(|n| n.stake.total_amount)
                        .sum();
                    (
                        0,
                        0,
                        validator_details.nominations.len(),
                        inactive_nomination_total,
                    )
                };

                let self_stake_formatted = format_decimal(
                    self_stake,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                );
                let active_nomination_formatted = format_decimal(
                    active_nomination_total,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                );
                let inactive_nomination_formatted = format_decimal(
                    inactive_nomination_total,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                );
                let validator_display = validator_details
                    .account
                    .get_display_or_condensed_address(None);
                context.insert("validator_display", &validator_display);
                context.insert("token_ticker", &CONFIG.substrate.token_ticker);
                context.insert("self_stake", &self_stake_formatted);
                context.insert("active_nomination_total", &active_nomination_formatted);
                context.insert("active_nominator_count", &active_nominator_count);
                context.insert("inactive_nomination_total", &inactive_nomination_formatted);
                context.insert("inactive_nominator_count", &inactive_nominator_count);
                "nomination_summary.html"
            }
            Self::NominationDetails {
                validator_details,
                onekv_nominator_account_ids,
            } => {
                let self_stake = validator_details.self_stake.total_amount;
                let self_stake_formatted = format_decimal(
                    self_stake,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                );
                let validator_display = validator_details
                    .account
                    .get_display_or_condensed_address(Some(3));
                context.insert("validator_display", &validator_display);
                context.insert("token_ticker", &CONFIG.substrate.token_ticker);
                context.insert("self_stake", &self_stake_formatted);
                let mut active_nominator_account_ids = Vec::new();
                if let Some(active_stake) = &validator_details.validator_stake {
                    let mut active_nomination_total = 0;
                    let active_nominations: Vec<(String, String, bool)> = active_stake
                        .nominators
                        .iter()
                        .map(|n| {
                            active_nomination_total += n.stake;
                            active_nominator_account_ids.push(n.account.id);
                            (
                                n.account.get_display_or_condensed_address(Some(3)),
                                n.stake,
                                onekv_nominator_account_ids.contains(&n.account.id),
                            )
                        })
                        .sorted_by(|n1, n2| n2.1.cmp(&n1.1))
                        .map(|n| {
                            (
                                n.0,
                                format_decimal(
                                    n.1,
                                    CONFIG.substrate.token_decimals,
                                    CONFIG.substrate.token_format_decimal_points,
                                ),
                                n.2,
                            )
                        })
                        .collect();
                    let max_len = active_nominations.get(0).map(|n| n.1.len()).unwrap_or(0);
                    context.insert(
                        "active_nomination_total",
                        &format_decimal(
                            active_nomination_total,
                            CONFIG.substrate.token_decimals,
                            CONFIG.substrate.token_format_decimal_points,
                        ),
                    );
                    context.insert(
                        "active_nominations",
                        &active_nominations
                            .iter()
                            .map(|n| {
                                (
                                    n.0.clone(),
                                    format!("{}{}", " ".repeat(max_len - n.1.len()), n.1),
                                    n.2,
                                )
                            })
                            .collect::<Vec<(String, String, bool)>>(),
                    );
                }
                let mut inactive_nomination_total = 0;
                let inactive_nominations: Vec<(String, String, bool)> = validator_details
                    .nominations
                    .iter()
                    .filter(|n| !active_nominator_account_ids.contains(&n.stash_account.id))
                    .map(|n| {
                        inactive_nomination_total += n.stake.active_amount;
                        (
                            n.stash_account.get_display_or_condensed_address(Some(3)),
                            n.stake.active_amount,
                            onekv_nominator_account_ids.contains(&n.stash_account.id),
                        )
                    })
                    .sorted_by(|n1, n2| n2.1.cmp(&n1.1))
                    .map(|n| {
                        (
                            n.0,
                            format_decimal(
                                n.1,
                                CONFIG.substrate.token_decimals,
                                CONFIG.substrate.token_format_decimal_points,
                            ),
                            n.2,
                        )
                    })
                    .collect();
                if !inactive_nominations.is_empty() {
                    let max_len = inactive_nominations.get(0).map(|n| n.1.len()).unwrap_or(0);
                    context.insert(
                        "inactive_nomination_total",
                        &format_decimal(
                            inactive_nomination_total,
                            CONFIG.substrate.token_decimals,
                            CONFIG.substrate.token_format_decimal_points,
                        ),
                    );
                    context.insert(
                        "inactive_nominations",
                        &inactive_nominations
                            .iter()
                            .map(|n| {
                                (
                                    n.0.clone(),
                                    format!("{}{}", " ".repeat(max_len - n.1.len()), n.1),
                                    n.2,
                                )
                            })
                            .collect::<Vec<(String, String, bool)>>(),
                    );
                }
                "nomination_details.html"
            }
            Self::ValidatorRemoved(validator) => {
                let display = if let Some(display) = &validator.display {
                    display.clone()
                } else {
                    get_condensed_address(&validator.address, None)
                };
                context.insert("display", &display);
                "validator_removed.html"
            }
            Self::Settings => "settings.html",
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

    fn get_settings_keyboard(&self) -> anyhow::Result<ReplyMarkup> {
        let rows = vec![
            vec![InlineKeyboardButton {
                text: self
                    .renderer
                    .render("settings_validator_activity.html", &Context::new())?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::SettingsValidatorActivity,
                    parameter: None,
                })?),
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }],
            vec![InlineKeyboardButton {
                text: self
                    .renderer
                    .render("settings_nominations.html", &Context::new())?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::SettingsNominations,
                    parameter: None,
                })?),
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }],
            vec![InlineKeyboardButton {
                text: self
                    .renderer
                    .render("settings_onekv.html", &Context::new())?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::SettingsOneKV,
                    parameter: None,
                })?),
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }],
            vec![InlineKeyboardButton {
                text: self
                    .renderer
                    .render("settings_democracy.html", &Context::new())?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::SettingsDemocracy,
                    parameter: None,
                })?),
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }],
            vec![InlineKeyboardButton {
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
            }],
        ];
        Ok(ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
            inline_keyboard: rows,
        }))
    }

    pub async fn send_message(
        &self,
        chat_id: i64,
        message_type: MessageType,
    ) -> anyhow::Result<MethodResponse<Message>> {
        let inline_keyboard = match &message_type {
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
                            query_type: query_type.clone(),
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
                        callback_data: Some(serde_json::to_string(&Query::get_cancel_query())?),
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
            MessageType::Settings => Some(self.get_settings_keyboard()?),
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
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }
}

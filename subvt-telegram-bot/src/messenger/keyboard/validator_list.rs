//! Validator selection keyboard - utilized by many other commands where a display of a list of
//! validators and the selection of one of them is required.
use crate::query::QueryType;
use crate::Query;
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use subvt_types::telegram::TelegramChatValidator;
use subvt_utility::text::get_condensed_address;
use tera::{Context, Tera};

pub fn get_validator_list_keyboard(
    renderer: &Tera,
    validators: &[TelegramChatValidator],
    query_type: &QueryType,
) -> anyhow::Result<Option<ReplyMarkup>> {
    if validators.is_empty() {
        Ok(None)
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
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                switch_inline_query_chosen_chat: None,
                callback_game: None,
                pay: None,
            }]);
        }
        rows.push(vec![InlineKeyboardButton {
            text: renderer.render("cancel.html", &Context::new())?,
            url: None,
            login_url: None,
            callback_data: Some(serde_json::to_string(&Query {
                query_type: QueryType::Cancel,
                parameter: None,
            })?),
            web_app: None,
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            switch_inline_query_chosen_chat: None,
            callback_game: None,
            pay: None,
        }]);
        Ok(Some(ReplyMarkup::InlineKeyboardMarkup(
            InlineKeyboardMarkup {
                inline_keyboard: rows,
            },
        )))
    }
}

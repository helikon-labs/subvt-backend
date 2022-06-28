//! Keyboard for the confirmation of the `/broadcast` request.
use crate::query::QueryType;
use crate::Query;
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};

pub fn get_broadcast_confirm_keyboard() -> anyhow::Result<Option<ReplyMarkup>> {
    let rows = vec![
        vec![InlineKeyboardButton {
            text: "Yes".to_string(),
            url: None,
            login_url: None,
            callback_data: Some(serde_json::to_string(&Query {
                query_type: QueryType::ConfirmBroadcast,
                parameter: None,
            })?),
            web_app: None,
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
            web_app: None,
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            callback_game: None,
            pay: None,
        }],
    ];
    Ok(Some(ReplyMarkup::InlineKeyboardMarkup(
        InlineKeyboardMarkup {
            inline_keyboard: rows,
        },
    )))
}

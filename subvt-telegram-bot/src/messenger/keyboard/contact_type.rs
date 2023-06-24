//! Keyboard for the selection of the `/contact` type: bug report or feature request.
use crate::query::QueryType;
use crate::Query;
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use tera::{Context, Tera};

pub fn get_contact_type_keyboard(renderer: &Tera) -> anyhow::Result<Option<ReplyMarkup>> {
    let rows = vec![
        vec![
            InlineKeyboardButton {
                text: renderer.render("report_bug.html", &Context::new())?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::ReportBug,
                    parameter: None,
                })?),
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                switch_inline_query_chosen_chat: None,
                callback_game: None,
                pay: None,
            },
            InlineKeyboardButton {
                text: renderer.render("report_feature_request.html", &Context::new())?,
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&Query {
                    query_type: QueryType::ReportFeatureRequest,
                    parameter: None,
                })?),
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                switch_inline_query_chosen_chat: None,
                callback_game: None,
                pay: None,
            },
        ],
        vec![InlineKeyboardButton {
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
        }],
    ];
    Ok(Some(ReplyMarkup::InlineKeyboardMarkup(
        InlineKeyboardMarkup {
            inline_keyboard: rows,
        },
    )))
}

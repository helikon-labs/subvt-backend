use crate::query::QueryType;
use crate::{Messenger, Query};
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use tera::Context;

impl Messenger {
    pub fn get_contact_type_keyboard(&self) -> anyhow::Result<Option<ReplyMarkup>> {
        let rows = vec![
            vec![
                InlineKeyboardButton {
                    text: self.renderer.render("report_bug.html", &Context::new())?,
                    url: None,
                    login_url: None,
                    callback_data: Some(serde_json::to_string(&Query {
                        query_type: QueryType::ReportBug,
                        parameter: None,
                    })?),
                    web_app: None,
                    switch_inline_query: None,
                    switch_inline_query_current_chat: None,
                    callback_game: None,
                    pay: None,
                },
                InlineKeyboardButton {
                    text: self
                        .renderer
                        .render("report_feature_request.html", &Context::new())?,
                    url: None,
                    login_url: None,
                    callback_data: Some(serde_json::to_string(&Query {
                        query_type: QueryType::ReportFeatureRequest,
                        parameter: None,
                    })?),
                    web_app: None,
                    switch_inline_query: None,
                    switch_inline_query_current_chat: None,
                    callback_game: None,
                    pay: None,
                },
            ],
            vec![InlineKeyboardButton {
                text: self.renderer.render("cancel.html", &Context::new())?,
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
}

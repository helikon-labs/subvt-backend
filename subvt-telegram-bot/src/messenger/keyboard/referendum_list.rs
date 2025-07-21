//! This keyboard lists open referenda. Every button is an open referendum, and its details is
//! displayed on a click on it.
use crate::query::QueryType;
use crate::Query;
use frankenstein::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use subvt_types::governance::polkassembly::ReferendumPost;
use tera::{Context, Tera};

pub fn get_referendum_list_keyboard(
    renderer: &Tera,
    track_id: u16,
    posts: &[ReferendumPost],
) -> anyhow::Result<Option<ReplyMarkup>> {
    if posts.is_empty() {
        Ok(None)
    } else {
        let mut rows = vec![];
        for post in posts {
            let params = (track_id, post.post_id.to_string());
            let query = Query {
                query_type: QueryType::ReferendumDetails,
                parameter: Some(serde_json::to_string(&params)?),
            };
            rows.push(vec![InlineKeyboardButton {
                text: format!(
                    "#{} - {}",
                    post.post_id,
                    if let Some(title) = post.maybe_title.as_ref().filter(|t| !t.is_empty()) {
                        title.to_owned()
                    } else if let Some(method) =
                        post.maybe_method.as_ref().filter(|m| !m.is_empty())
                    {
                        method.to_owned()
                    } else {
                        renderer.render("no_title.html", &Context::new())?
                    },
                ),
                url: None,
                login_url: None,
                callback_data: Some(serde_json::to_string(&query)?),
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                switch_inline_query_chosen_chat: None,
                callback_game: None,
                pay: None,
                copy_text: None,
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
            copy_text: None,
        }]);
        Ok(Some(ReplyMarkup::InlineKeyboardMarkup(
            InlineKeyboardMarkup {
                inline_keyboard: rows,
            },
        )))
    }
}

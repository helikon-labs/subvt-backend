use crate::query::QueryType;
use crate::{Messenger, Query};
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use subvt_types::governance::polkassembly::ReferendumPost;
use tera::Context;

impl Messenger {
    pub fn get_referendum_list_keyboard(
        &self,
        posts: &[ReferendumPost],
    ) -> anyhow::Result<Option<ReplyMarkup>> {
        if posts.is_empty() {
            Ok(None)
        } else {
            let mut rows = vec![];
            for post in posts {
                let query = Query {
                    query_type: QueryType::ReferendumDetails,
                    parameter: Some(post.onchain_link.onchain_referendum_id.to_string()),
                };
                rows.push(vec![InlineKeyboardButton {
                    text: format!(
                        "#{} - {}",
                        post.onchain_link.onchain_referendum_id,
                        if let Some(title) = &post.maybe_title {
                            title.to_owned()
                        } else {
                            self.renderer.render("no_title.html", &Context::new())?
                        },
                    ),
                    url: None,
                    login_url: None,
                    callback_data: Some(serde_json::to_string(&query)?),
                    web_app: None,
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
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
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
}

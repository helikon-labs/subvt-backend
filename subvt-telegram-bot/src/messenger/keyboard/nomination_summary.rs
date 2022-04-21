use crate::query::QueryType;
use crate::{Messenger, Query};
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use subvt_types::subvt::ValidatorDetails;
use tera::Context;

impl Messenger {
    pub fn get_nomination_summary_keyboard(
        &self,
        chat_validator_id: u64,
        validator_details: &ValidatorDetails,
    ) -> anyhow::Result<Option<ReplyMarkup>> {
        if validator_details.nominations.is_empty() {
            Ok(None)
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
                web_app: None,
                switch_inline_query: None,
                switch_inline_query_current_chat: None,
                callback_game: None,
                pay: None,
            }]];
            Ok(Some(ReplyMarkup::InlineKeyboardMarkup(
                InlineKeyboardMarkup {
                    inline_keyboard: rows,
                },
            )))
        }
    }
}

//! Single-button keyboard to move on to the nomination details from the nomination summary
//! for a validator.
use crate::query::QueryType;
use crate::Query;
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use subvt_types::subvt::ValidatorDetails;
use tera::{Context, Tera};

pub fn get_nomination_summary_keyboard(
    renderer: &Tera,
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
            text: renderer.render("view_nomination_details.html", &Context::new())?,
            url: None,
            login_url: None,
            callback_data: Some(serde_json::to_string(&query)?),
            web_app: None,
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            switch_inline_query_chosen_chat: None,
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

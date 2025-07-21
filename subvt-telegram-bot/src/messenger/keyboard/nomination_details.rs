use crate::query::QueryType;
use crate::Query;
use frankenstein::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use subvt_types::subvt::ValidatorDetails;
use tera::{Context, Tera};

pub fn get_nomination_details_keyboard(
    renderer: &Tera,
    chat_validator_id: u64,
    validator_details: &ValidatorDetails,
    is_full: bool,
) -> anyhow::Result<Option<ReplyMarkup>> {
    if is_full || validator_details.nominations.is_empty() {
        Ok(None)
    } else {
        let query = Query {
            query_type: QueryType::NominationDetailsFull,
            parameter: Some(chat_validator_id.to_string()),
        };
        let rows = vec![vec![InlineKeyboardButton {
            text: renderer.render("view_full_nomination_details.html", &Context::new())?,
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
        }]];
        Ok(Some(ReplyMarkup::InlineKeyboardMarkup(
            InlineKeyboardMarkup {
                inline_keyboard: rows,
            },
        )))
    }
}

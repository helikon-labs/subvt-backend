use crate::query::{Query, QueryType};
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use subvt_types::governance::track::Track;
use tera::{Context, Tera};

pub fn get_referendum_tracks_keyboard(
    renderer: &Tera,
    data: &Vec<(Track, usize)>,
) -> anyhow::Result<Option<ReplyMarkup>> {
    let mut rows = vec![];
    for (track, count) in data {
        let query = Query {
            query_type: QueryType::ReferendumTracks,
            parameter: Some(track.id().to_string()),
        };
        rows.push(vec![InlineKeyboardButton {
            text: format!("{} ({})", track.name(), count),
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

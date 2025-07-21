//! NFT selection keyboard to visit an NFT's URL, displayed as response to the `/nfts` command.
use crate::messenger::button::settings::get_settings_button;
use crate::query::QueryType;
use crate::Query;
use frankenstein::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use itertools::Itertools;
use subvt_types::sub_id::NFTCollection;
use tera::{Context, Tera};

pub fn get_nft_collection_keyboard(
    renderer: &Tera,
    validator_id: u64,
    collection_page: &NFTCollection,
    page_index: usize,
    has_prev: bool,
    has_next: bool,
) -> anyhow::Result<InlineKeyboardMarkup> {
    let sorted_chain_keys = collection_page
        .keys()
        .sorted_by_key(|chain| chain.name())
        .collect_vec();
    let mut rows = vec![];
    for chain in sorted_chain_keys {
        if let Some(chain_collection) = collection_page.get(chain) {
            for nft in chain_collection {
                let url = if let Some(url) = &nft.url {
                    Some(url.clone())
                } else {
                    nft.image_url.clone()
                };
                rows.push(vec![InlineKeyboardButton {
                    text: format!(
                        "{} - {}",
                        chain.name(),
                        if let Some(name) = &nft.name {
                            name
                        } else {
                            &nft.id
                        }
                    ),
                    url: url.clone(),
                    login_url: None,
                    callback_data: if url.is_none() {
                        Some(serde_json::to_string(&Query {
                            query_type: QueryType::NoOp,
                            parameter: None,
                        })?)
                    } else {
                        None
                    },
                    web_app: None,
                    switch_inline_query: None,
                    switch_inline_query_current_chat: None,
                    switch_inline_query_chosen_chat: None,
                    callback_game: None,
                    pay: None,
                    copy_text: None,
                }]);
            }
        }
    }
    if has_prev || has_next {
        let mut nav_rows = vec![];
        nav_rows.push(InlineKeyboardButton {
            text: if has_prev {
                let mut context = Context::new();
                context.insert("page_number", &(page_index));
                renderer.render("prev_page.html", &context)?
            } else {
                "•".to_string()
            },
            url: None,
            login_url: None,
            callback_data: if has_prev {
                Some(serde_json::to_string(&Query {
                    query_type: QueryType::NFTs(page_index - 1, false),
                    parameter: Some(validator_id.to_string()),
                })?)
            } else {
                Some(serde_json::to_string(&Query {
                    query_type: QueryType::NoOp,
                    parameter: None,
                })?)
            },
            web_app: None,
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            switch_inline_query_chosen_chat: None,
            callback_game: None,
            pay: None,
            copy_text: None,
        });
        nav_rows.push(InlineKeyboardButton {
            text: if has_next {
                let mut context = Context::new();
                context.insert("page_number", &(page_index + 2));
                renderer.render("next_page.html", &context)?
            } else {
                "•".to_string()
            },
            url: None,
            login_url: None,
            callback_data: if has_next {
                Some(serde_json::to_string(&Query {
                    query_type: QueryType::NFTs(page_index + 1, false),
                    parameter: Some(validator_id.to_string()),
                })?)
            } else {
                Some(serde_json::to_string(&Query {
                    query_type: QueryType::NoOp,
                    parameter: None,
                })?)
            },
            web_app: None,
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            switch_inline_query_chosen_chat: None,
            callback_game: None,
            pay: None,
            copy_text: None,
        });
        rows.push(nav_rows);
    }
    rows.push(get_settings_button(
        renderer,
        "close.html",
        QueryType::Close,
    )?);
    Ok(InlineKeyboardMarkup {
        inline_keyboard: rows,
    })
}

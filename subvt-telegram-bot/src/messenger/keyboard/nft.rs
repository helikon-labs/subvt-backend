use crate::query::QueryType;
use crate::{Messenger, Query};
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};
use itertools::Itertools;
use subvt_types::sub_id::NFTCollection;
use tera::Context;

impl Messenger {
    pub fn get_nft_collection_keyboard(
        &self,
        collection: &NFTCollection,
        page_index: usize,
        has_prev: bool,
        has_next: bool,
    ) -> anyhow::Result<Option<ReplyMarkup>> {
        let sorted_chain_keys = collection
            .keys()
            .sorted_by_key(|chain| chain.name())
            .collect_vec();
        let mut rows = vec![];
        for chain in sorted_chain_keys {
            if let Some(chain_collection) = collection.get(chain) {
                for nft in chain_collection {
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
                        url: nft.url.clone(),
                        login_url: None,
                        callback_data: None,
                        web_app: None,
                        switch_inline_query: None,
                        switch_inline_query_current_chat: None,
                        callback_game: None,
                        pay: None,
                    }]);
                }
            }
        }
        if has_next || has_prev {
            let mut nav_rows = vec![];
            if has_prev {
                let mut context = Context::new();
                context.insert("page_number", &(page_index));
                nav_rows.push(InlineKeyboardButton {
                    text: self.renderer.render("prev_page.html", &context)?,
                    url: None,
                    login_url: None,
                    callback_data: Some(serde_json::to_string(&Query {
                        query_type: QueryType::NFTs(page_index - 1),
                        parameter: None,
                    })?),
                    web_app: None,
                    switch_inline_query: None,
                    switch_inline_query_current_chat: None,
                    callback_game: None,
                    pay: None,
                });
            }
            if has_next {
                let mut context = Context::new();
                context.insert("page_number", &(page_index + 2));
                nav_rows.push(InlineKeyboardButton {
                    text: self.renderer.render("next_page.html", &context)?,
                    url: None,
                    login_url: None,
                    callback_data: Some(serde_json::to_string(&Query {
                        query_type: QueryType::NFTs(page_index + 1),
                        parameter: None,
                    })?),
                    web_app: None,
                    switch_inline_query: None,
                    switch_inline_query_current_chat: None,
                    callback_game: None,
                    pay: None,
                });
            }
            rows.push(nav_rows);
        }
        rows.push(self.get_settings_button("close.html", QueryType::Close)?);
        Ok(Some(ReplyMarkup::InlineKeyboardMarkup(
            InlineKeyboardMarkup {
                inline_keyboard: rows,
            },
        )))
    }
}

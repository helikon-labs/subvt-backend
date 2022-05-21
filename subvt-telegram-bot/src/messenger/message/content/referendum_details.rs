//! Content for a selected open referendum.
use crate::MessageType;
use crate::CONFIG;
use subvt_types::app::event::democracy::DemocracyVotedEvent;
use subvt_types::governance::polkassembly::ReferendumPost;
use subvt_types::telegram::TelegramChatValidator;
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::get_condensed_address;
use tera::Context;

impl MessageType {
    pub(crate) fn fill_referendum_details_context(
        &self,
        context: &mut Context,
        post: &ReferendumPost,
        chat_validators: &[TelegramChatValidator],
        validator_votes: &[DemocracyVotedEvent],
    ) {
        context.insert("chain", &CONFIG.substrate.chain);
        context.insert("referendum_id", &post.onchain_link.onchain_referendum_id);
        if let Some(title) = &post.maybe_title {
            context.insert("title", title);
        }
        context.insert("proposer_address", &post.onchain_link.proposer_address);
        context.insert(
            "condensed_proposer_address",
            &get_condensed_address(&post.onchain_link.proposer_address, None),
        );
        let referendum = &post.onchain_link.onchain_referendum[0];
        context.insert("vote_threshold", &referendum.vote_threshold);
        context.insert("end_block_number", &referendum.end_block_number);
        if let Some(status) = referendum.referendum_status.last() {
            context.insert("status", &status.status);
        }
        if let Some(content) = &post.maybe_content {
            context.insert("content", &content);
        }

        // validator votes
        let mut validators_without_vote = vec![];
        let mut context_validator_votes = vec![];
        for validator in chat_validators {
            if let Some(validator_vote) = validator_votes
                .iter()
                .find(|vote| vote.account_id == validator.account_id)
            {
                context_validator_votes.push((
                    validator
                        .display
                        .clone()
                        .unwrap_or_else(|| get_condensed_address(&validator.address, None)),
                    if let Some(balance) = validator_vote.aye_balance {
                        format_decimal(
                            balance,
                            CONFIG.substrate.token_decimals,
                            CONFIG.substrate.token_format_decimal_points,
                        )
                    } else {
                        "".to_string()
                    },
                    if let Some(balance) = validator_vote.nay_balance {
                        format_decimal(
                            balance,
                            CONFIG.substrate.token_decimals,
                            CONFIG.substrate.token_format_decimal_points,
                        )
                    } else {
                        "".to_string()
                    },
                    if let Some(conviction) = validator_vote.conviction {
                        conviction
                    } else {
                        0
                    },
                ));
            } else {
                validators_without_vote.push(
                    validator
                        .display
                        .clone()
                        .unwrap_or_else(|| get_condensed_address(&validator.address, None)),
                );
            };
        }
        context.insert("token_ticker", &CONFIG.substrate.token_ticker);
        context.insert("validator_votes", &context_validator_votes);
        context.insert("validators_without_vote", &validators_without_vote);
    }
}

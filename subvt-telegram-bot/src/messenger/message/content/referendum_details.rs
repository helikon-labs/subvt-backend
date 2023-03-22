//! Content for a selected open referendum.
use crate::MessageType;
use crate::CONFIG;
use subvt_types::governance::polkassembly::ReferendumPostDetails;
use subvt_types::substrate::democracy::ReferendumVote;
use subvt_types::telegram::TelegramChatValidator;
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::get_condensed_address;
use tera::Context;

impl MessageType {
    pub(in crate::messenger::message) fn fill_referendum_details_context(
        &self,
        context: &mut Context,
        post: &ReferendumPostDetails,
        chat_validator_votes: &[(TelegramChatValidator, Option<ReferendumVote>)],
    ) {
        context.insert("chain", &CONFIG.substrate.chain);
        context.insert("referendum_id", &post.post_id);
        if let Some(title) = &post.maybe_title {
            context.insert("title", title);
        }
        context.insert("proposer_address", &post.proposer);
        context.insert(
            "condensed_proposer_address",
            &get_condensed_address(&post.proposer, None),
        );
        context.insert("vote_threshold", &post.vote_threshold);
        context.insert("end_block_number", &post.end_block_number);
        context.insert("status", &post.status);
        if let Some(content) = &post.maybe_content {
            context.insert("content", &content);
        }

        // validator votes
        let mut validators_without_vote = vec![];
        let mut context_validator_votes = vec![];
        for validator_vote in chat_validator_votes {
            if let Some(vote) = validator_vote.1 {
                if let Some(direct_vote) = vote.direct_vote {
                    context_validator_votes.push((
                        validator_vote.0.display.clone().unwrap_or_else(|| {
                            get_condensed_address(&validator_vote.0.address, None)
                        }),
                        false,
                        if let Some(balance) = direct_vote.aye {
                            format_decimal(
                                balance,
                                CONFIG.substrate.token_decimals,
                                CONFIG.substrate.token_format_decimal_points,
                            )
                        } else {
                            "".to_string()
                        },
                        if let Some(balance) = direct_vote.nay {
                            format_decimal(
                                balance,
                                CONFIG.substrate.token_decimals,
                                CONFIG.substrate.token_format_decimal_points,
                            )
                        } else {
                            "".to_string()
                        },
                        if let Some(conviction) = direct_vote.conviction {
                            conviction
                        } else {
                            0
                        },
                    ));
                } else if let Some(delegated_vote) = vote.delegated_vote {
                    context_validator_votes.push((
                        validator_vote.0.display.clone().unwrap_or_else(|| {
                            get_condensed_address(&validator_vote.0.address, None)
                        }),
                        true,
                        if delegated_vote.vote.aye.is_some() {
                            format_decimal(
                                delegated_vote.balance,
                                CONFIG.substrate.token_decimals,
                                CONFIG.substrate.token_format_decimal_points,
                            )
                        } else {
                            "".to_string()
                        },
                        if delegated_vote.vote.nay.is_some() {
                            format_decimal(
                                delegated_vote.balance,
                                CONFIG.substrate.token_decimals,
                                CONFIG.substrate.token_format_decimal_points,
                            )
                        } else {
                            "".to_string()
                        },
                        delegated_vote.conviction,
                    ));
                } else {
                    validators_without_vote.push(
                        validator_vote.0.display.clone().unwrap_or_else(|| {
                            get_condensed_address(&validator_vote.0.address, None)
                        }),
                    );
                }
            } else {
                validators_without_vote.push(
                    validator_vote
                        .0
                        .display
                        .clone()
                        .unwrap_or_else(|| get_condensed_address(&validator_vote.0.address, None)),
                );
            };
        }
        context.insert("token_ticker", &CONFIG.substrate.token_ticker);
        context.insert("validator_votes", &context_validator_votes);
        context.insert("validators_without_vote", &validators_without_vote);
    }
}

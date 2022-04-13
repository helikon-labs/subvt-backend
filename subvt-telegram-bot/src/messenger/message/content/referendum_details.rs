use crate::MessageType;
use crate::CONFIG;
use subvt_types::governance::polkassembly::ReferendumPost;
use subvt_utility::text::get_condensed_address;
use tera::Context;

impl MessageType {
    pub(crate) fn fill_referendum_details_context(
        &self,
        context: &mut Context,
        post: &ReferendumPost,
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
    }
}

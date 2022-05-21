//! Content for the `/summary` request, displays a summary of all validators.
use crate::{MessageType, CONFIG};
use subvt_types::telegram::TelegramChatValidatorSummary;
use tera::Context;

impl MessageType {
    pub(crate) fn fill_validators_summary_context(
        &self,
        context: &mut Context,
        validator_summaries: &Vec<TelegramChatValidatorSummary>,
    ) {
        context.insert("chain", &CONFIG.substrate.chain);
        context.insert("token_ticker", &CONFIG.substrate.token_ticker);
        context.insert("validator_summaries", validator_summaries);
    }
}

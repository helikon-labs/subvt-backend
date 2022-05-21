//! `/summary` command processor.
use crate::{MessageType, TelegramBot, CONFIG};
use subvt_governance::polkassembly::fetch_open_referendum_list;
use subvt_types::telegram::TelegramChatValidatorSummary;

impl TelegramBot {
    //! Sends the user a handy summary of all added validators.
    pub(crate) async fn process_summary_command(&self, chat_id: i64) -> anyhow::Result<()> {
        let chat_validators = self.network_postgres.get_chat_validators(chat_id).await?;
        if chat_validators.is_empty() {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::NoValidatorsOnChat),
                )
                .await?;
            return Ok(());
        }
        // get referenda
        let open_referenda = fetch_open_referendum_list().await?;
        let mut chat_validator_summaries: Vec<TelegramChatValidatorSummary> = vec![];
        for chat_validator in chat_validators {
            if let Some(validator_details) = &self
                .redis
                .fetch_validator_details(&chat_validator.account_id)
                .await?
            {
                let mut validator_summary = TelegramChatValidatorSummary::from(
                    validator_details,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                );
                for open_referendum in &open_referenda {
                    if self
                        .network_postgres
                        .get_account_vote_for_referendum(
                            &chat_validator.account_id,
                            open_referendum.onchain_link.onchain_referendum_id,
                        )
                        .await?
                        .is_none()
                    {
                        validator_summary
                            .missing_referendum_votes
                            .push(open_referendum.onchain_link.onchain_referendum_id);
                    }
                }
                validator_summary.missing_referendum_votes.sort_unstable();
                chat_validator_summaries.push(validator_summary);
            }
        }
        self.messenger
            .send_message(
                &self.app_postgres,
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::ValidatorsSummary(chat_validator_summaries)),
            )
            .await?;
        Ok(())
    }
}

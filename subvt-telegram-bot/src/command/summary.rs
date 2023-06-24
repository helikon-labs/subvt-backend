//! `/summary` command processor.
use crate::{MessageType, Messenger, TelegramBot, CONFIG};
use subvt_substrate_client::SubstrateClient;
use subvt_types::telegram::TelegramChatValidatorSummary;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
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
        let referenda = self.network_postgres.get_open_referenda(None).await?;
        let mut chat_validator_summaries: Vec<TelegramChatValidatorSummary> = vec![];
        let substrate_client = SubstrateClient::new(&CONFIG).await?;
        for chat_validator in chat_validators {
            if let Some(validator_details) = &self
                .redis
                .fetch_validator_details(
                    self.redis.get_finalized_block_summary().await?.number,
                    &chat_validator.account_id,
                )
                .await?
            {
                let mut validator_summary = TelegramChatValidatorSummary::from(
                    validator_details,
                    CONFIG.substrate.token_decimals,
                    CONFIG.substrate.token_format_decimal_points,
                );
                for referendum in &referenda {
                    if substrate_client
                        .get_account_referendum_conviction_vote(
                            &chat_validator.account_id,
                            referendum.track_id,
                            referendum.post_id,
                            None,
                        )
                        .await?
                        .is_none()
                    {
                        validator_summary
                            .missing_referendum_votes
                            .push(referendum.post_id);
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

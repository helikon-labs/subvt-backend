use crate::query::Query;
use crate::{messenger::message::MessageType, TelegramBot};
use subvt_utility::text::get_condensed_address;

impl TelegramBot {
    pub(crate) async fn process_rewards_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
        query: &Query,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        if let Some(id_str) = &query.parameter {
            log::info!("Validator selected for rewards in chat {}.", chat_id);
            if let Some(validator) = self
                .network_postgres
                .get_chat_validator_by_id(chat_id, id_str.parse()?)
                .await?
            {
                let era_rewards = self
                    .network_postgres
                    .get_validator_era_rewards(&validator.account_id)
                    .await?;
                if era_rewards.is_empty() {
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::NoRewardsFound),
                        )
                        .await?;
                } else {
                    let title = format!(
                        "Monthly Staking Rewards for {}",
                        get_condensed_address(&validator.address, Some(3)),
                    );
                    let path = subvt_plotter::rewards::plot_era_rewards(&title, &era_rewards)?;
                    self.messenger
                        .send_image(&self.app_postgres, &self.network_postgres, chat_id, &path)
                        .await?;
                    if let Err(error) = std::fs::remove_file(&path) {
                        log::error!("Error while removing payout report PNG file: {:?}", error);
                    }
                }
            } else {
                self.messenger
                    .send_message(
                        &self.app_postgres,
                        &self.network_postgres,
                        chat_id,
                        Box::new(MessageType::ValidatorNotFound {
                            maybe_address: None,
                        }),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

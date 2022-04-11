use crate::query::Query;
use crate::{messenger::message::MessageType, TelegramBot};
use chrono::Datelike;
use subvt_types::substrate::Balance;

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
                    .get_validator_rewards(&validator.account_id)
                    .await?;
                if era_rewards.is_empty() {
                    println!("no rewards for {}", validator.address);
                } else {
                    println!("send rewards report for {}", validator.address);
                    // (month, year, reward)
                    let mut monthly_rewards: Vec<(u8, u32, Balance)> = Vec::new();
                    for (era, reward) in era_rewards {
                        let month = era.get_start_date_time().month() as u8;
                        let year = era.get_start_date_time().year() as u32;
                        let last = monthly_rewards.last().unwrap_or(&(0, 0, 0));
                        if last.0 == month && last.1 == year {
                            let last_index = monthly_rewards.len() - 1;
                            monthly_rewards[last_index] = (last.0, last.1, last.2 + reward);
                        } else {
                            monthly_rewards.push((month, year, reward));
                        }
                    }
                    subvt_plotter::plot_validator_monthly_rewards(&monthly_rewards);
                    self.messenger
                        .send_image(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            "/Users/kukabi/Desktop/chart.png",
                        )
                        .await?;
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

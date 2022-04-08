use crate::{MessageType, TelegramBot};

impl TelegramBot {
    pub(crate) async fn process_network_status_command(&self, chat_id: i64) -> anyhow::Result<()> {
        self.messenger
            .send_message(
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::NetworkStatus(
                    self.redis.get_current_network_status().await?,
                )),
            )
            .await?;
        Ok(())
    }
}

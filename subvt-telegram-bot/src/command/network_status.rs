//! `/networkstatus` command processor.
use crate::{MessageType, Messenger, TelegramBot};

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    //! Fetches the current live network status from the network Redis instance and
    //! sends it to the chat.
    pub(crate) async fn process_network_status_command(&self, chat_id: i64) -> anyhow::Result<()> {
        self.messenger
            .send_message(
                &self.app_postgres,
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::NetworkStatus(
                    self.redis.get_network_status().await?,
                )),
            )
            .await?;
        Ok(())
    }
}

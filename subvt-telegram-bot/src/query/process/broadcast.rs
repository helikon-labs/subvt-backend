use crate::{messenger::message::MessageType, TelegramBot};

impl TelegramBot {
    pub(crate) async fn process_confirm_broadcast_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        log::info!("Broadcast confirmed, sending.");
        for chat_id in self.network_postgres.get_chat_ids().await? {
            match self
                .messenger
                .send_message(
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::Broadcast),
                )
                .await
            {
                Ok(_) => log::info!("Broadcast sent to chat {}.", chat_id),
                Err(error) => log::error!(
                    "Error while sending broadcast to chat {}: {:?}",
                    chat_id,
                    error
                ),
            }
        }
        Ok(())
    }
}

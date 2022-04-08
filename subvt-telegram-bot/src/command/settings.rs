use crate::{MessageType, TelegramBot};

impl TelegramBot {
    pub(crate) async fn process_settings_command(&self, chat_id: i64) -> anyhow::Result<()> {
        // close last settings message
        if let Some(settings_message_id) = self
            .network_postgres
            .get_chat_settings_message_id(chat_id)
            .await?
        {
            let _ = self
                .messenger
                .delete_message(chat_id, settings_message_id)
                .await;
        }
        let settings_message_id = self
            .messenger
            .send_message(
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::Settings),
            )
            .await?
            .result
            .message_id;
        self.network_postgres
            .set_chat_settings_message_id(chat_id, Some(settings_message_id))
            .await?;
        Ok(())
    }
}

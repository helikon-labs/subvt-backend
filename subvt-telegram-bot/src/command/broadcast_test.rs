use crate::{MessageType, TelegramBot, CONFIG};

impl TelegramBot {
    pub(crate) async fn process_broadcast_test_command(
        &self,
        chat_id: i64,
        command: &str,
    ) -> anyhow::Result<()> {
        if CONFIG.telegram_bot.admin_chat_id == chat_id {
            self.messenger
                .send_message(chat_id, Box::new(MessageType::Broadcast))
                .await?;
        } else {
            self.messenger
                .send_message(
                    chat_id,
                    Box::new(MessageType::UnknownCommand(command.to_string())),
                )
                .await?;
        }
        Ok(())
    }
}

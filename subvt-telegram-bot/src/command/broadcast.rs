//! `/broadcast` command processor.
use crate::{MessageType, Messenger, TelegramBot, CONFIG};

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    //! This command is used by the chat admin(s) to broadcast messages to all chats.
    pub(crate) async fn process_broadcast_command(
        &self,
        chat_id: i64,
        command: &str,
    ) -> anyhow::Result<()> {
        if CONFIG.telegram_bot.get_admin_chat_ids().contains(&chat_id) {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::BroadcastConfirm),
                )
                .await?;
        } else {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::UnknownCommand(command.to_string())),
                )
                .await?;
        }
        Ok(())
    }
}

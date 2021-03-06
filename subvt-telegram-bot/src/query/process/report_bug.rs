use crate::{MessageType, Messenger, TelegramBot};
use subvt_types::telegram::TelegramChatState;

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    pub(crate) async fn process_report_bug_query(
        &self,
        chat_id: i64,
        original_message_id: Option<i32>,
    ) -> anyhow::Result<()> {
        if let Some(message_id) = original_message_id {
            self.messenger.delete_message(chat_id, message_id).await?;
        }
        self.network_postgres
            .set_chat_state(chat_id, TelegramChatState::EnterBugReport)
            .await?;
        self.messenger
            .send_message(
                &self.app_postgres,
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::EnterBugReport),
            )
            .await?;
        Ok(())
    }
}

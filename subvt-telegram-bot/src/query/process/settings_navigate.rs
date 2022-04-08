use crate::query::SettingsSubSection;
use crate::TelegramBot;

impl TelegramBot {
    pub(crate) async fn process_settings_navigate_query(
        &self,
        chat_id: i64,
        sub_section: SettingsSubSection,
    ) -> anyhow::Result<()> {
        if let Some(settings_message_id) = self
            .network_postgres
            .get_chat_settings_message_id(chat_id)
            .await?
        {
            let user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
            let notification_rules = self
                .app_postgres
                .get_user_notification_rules(user_id)
                .await?;
            self.messenger
                .update_settings_message(
                    chat_id,
                    settings_message_id,
                    sub_section,
                    &notification_rules,
                )
                .await?;
        }
        Ok(())
    }
}

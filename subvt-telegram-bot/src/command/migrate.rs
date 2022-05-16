use crate::{MessageType, TelegramBot, CONFIG};
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{options::ClientOptions, Client};
use subvt_types::telegram::{OneKVBotChat, OneKVBotValidator, TelegramChatState};

const MIGRATION_CODE_LENGTH: usize = 5;

impl TelegramBot {
    pub(crate) async fn process_migrate_command(
        &self,
        chat_id: i64,
        args: &[String],
    ) -> anyhow::Result<()> {
        if args.is_empty() {
            self.network_postgres
                .set_chat_state(chat_id, TelegramChatState::EnterMigrationCode)
                .await?;
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::EnterMigrationCode),
                )
                .await?;
            return Ok(());
        }
        let migration_code: &str = args.get(0).unwrap();
        if migration_code.len() > MIGRATION_CODE_LENGTH {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::MigrationInvalidCode),
                )
                .await?;
            return Ok(());
        }
        let client_options = ClientOptions::parse(&CONFIG.telegram_bot.mongo_url).await?;
        let client = Client::with_options(client_options)?;
        let onekv_bot_db = client.database(&CONFIG.telegram_bot.mongo_db_name);
        let chat_collection = onekv_bot_db.collection::<OneKVBotChat>("chats");
        let chat = {
            let chat_filter = doc! { "migrationCode": migration_code };
            let mut chat_cursor = chat_collection.find(chat_filter, None).await?;
            match chat_cursor.try_next().await? {
                Some(chat) => chat,
                None => {
                    self.messenger
                        .send_message(
                            &self.app_postgres,
                            &self.network_postgres,
                            chat_id,
                            Box::new(MessageType::MigrationChatNotFound),
                        )
                        .await?;
                    return Ok(());
                }
            }
        };
        if chat.is_migrated.unwrap_or(false) {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::MigrationAlreadyMigrated),
                )
                .await?;
            return Ok(());
        }
        let validator_collection = onekv_bot_db.collection::<OneKVBotValidator>("validators");
        let validators = {
            let validator_filter = doc! { "chatIds": { "$elemMatch": { "$eq": chat.chat_id } } };
            let mut validator_cursor = validator_collection.find(validator_filter, None).await?;
            let mut validators = vec![];
            while let Some(validator) = validator_cursor.try_next().await? {
                validators.push(validator);
            }
            validators
        };
        if validators.is_empty() {
            self.messenger
                .send_message(
                    &self.app_postgres,
                    &self.network_postgres,
                    chat_id,
                    Box::new(MessageType::MigrationNoValidatorFound),
                )
                .await?;
            return Ok(());
        }
        /*
        chat.block_notification_period => off'sa off
        chat.send_chilling_event_notifications on off
        chat.send_new_nomination_notifications on off
        chat.send_offline_event_notifications on off
        chat.unclaimed_payout_notification_period db'yi incele
         */
        for validator in &validators {
            let stash_address = validator.stash_address.as_str();
            self.process_command(chat_id, "/add", &[stash_address.to_string()])
                .await?;
            let chat_ids: Vec<i64> = validator
                .chat_ids
                .iter()
                .filter(|validator_chat_id| **validator_chat_id != chat_id)
                .copied()
                .collect();
            validator_collection
                .update_one(
                    doc! { "stashAddress": stash_address },
                    doc! { "$set": { "chatIds": chat_ids } },
                    None,
                )
                .await?;
        }
        /*
        update notification settings
         */
        chat_collection
            .update_one(
                doc! { "chatId": chat_id },
                doc! { "$set": { "isMigrated": true } },
                None,
            )
            .await?;
        self.messenger
            .send_message(
                &self.app_postgres,
                &self.network_postgres,
                chat_id,
                Box::new(MessageType::MigrationSuccessful(validators.len())),
            )
            .await?;
        Ok(())
    }
}

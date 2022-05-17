use crate::{MessageType, TelegramBot, CONFIG};
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{options::ClientOptions, Client};
use subvt_types::app::{NotificationPeriodType, NotificationTypeCode};
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
        // migrate notification rules
        let user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
        // chilling events
        if !chat.send_chilling_event_notifications {
            self.app_postgres
                .update_user_notification_rule_period(
                    user_id,
                    NotificationTypeCode::ChainValidatorChilled,
                    NotificationPeriodType::Off,
                    0,
                )
                .await?;
        }
        // new & lost nominations
        if !chat.send_new_nomination_notifications {
            self.app_postgres
                .update_user_notification_rule_period(
                    user_id,
                    NotificationTypeCode::ChainValidatorNewNomination,
                    NotificationPeriodType::Off,
                    0,
                )
                .await?;
            self.app_postgres
                .update_user_notification_rule_period(
                    user_id,
                    NotificationTypeCode::ChainValidatorLostNomination,
                    NotificationPeriodType::Off,
                    0,
                )
                .await?;
        }
        // offline offences
        if !chat.send_offline_event_notifications {
            self.app_postgres
                .update_user_notification_rule_period(
                    user_id,
                    NotificationTypeCode::ChainValidatorOfflineOffence,
                    NotificationPeriodType::Off,
                    0,
                )
                .await?;
        }
        // block authorship
        let (block_notification_period_type, block_notification_period) =
            match chat.block_notification_period {
                -1 => (NotificationPeriodType::Off, 0),
                0 => (NotificationPeriodType::Immediate, 0),
                60 => (NotificationPeriodType::Hour, 1),
                180 | 720 => (NotificationPeriodType::Epoch, 3),
                360 | 1440 => (NotificationPeriodType::Era, 1),
                _ => (NotificationPeriodType::Hour, 1),
            };
        self.app_postgres
            .update_user_notification_rule_period(
                user_id,
                NotificationTypeCode::ChainValidatorBlockAuthorship,
                block_notification_period_type,
                block_notification_period,
            )
            .await?;
        // unclaimed payouts
        if chat.unclaimed_payout_notification_period == -1 {
            self.app_postgres
                .update_user_notification_rule_period(
                    user_id,
                    NotificationTypeCode::ChainValidatorUnclaimedPayout,
                    NotificationPeriodType::Off,
                    0,
                )
                .await?;
        }
        // migrate validators
        for validator in &validators {
            let stash_address = validator.stash_address.as_str();
            self.process_command(chat_id, "/add", &[stash_address.to_string()])
                .await?;
            let chat_ids: Vec<i64> = validator
                .chat_ids
                .iter()
                .filter(|validator_chat_id| **validator_chat_id != chat.chat_id)
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
        chat_collection
            .update_one(
                doc! { "chatId": chat.chat_id },
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

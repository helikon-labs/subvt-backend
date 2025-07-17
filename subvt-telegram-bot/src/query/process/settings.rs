use crate::query::{Query, SettingsEditQueryType, SettingsSubSection};
use crate::{Messenger, TelegramBot, CONFIG};
use rustc_hash::FxHashSet as HashSet;
use subvt_types::app::notification::{
    NotificationChannel, NotificationPeriodType, NotificationTypeCode,
};

impl<M: Messenger + Send + Sync> TelegramBot<M> {
    async fn create_rule_if_not_exists(
        &self,
        user_id: u32,
        type_code: NotificationTypeCode,
    ) -> anyhow::Result<()> {
        let user_notification_rules = self
            .app_postgres
            .get_user_notification_rules(user_id)
            .await?;
        let rule_exists = user_notification_rules
            .iter()
            .filter(|rule| rule.notification_type.code == type_code.to_string())
            .any(|rule| {
                rule.notification_channels
                    .iter()
                    .any(|channel| channel.channel == NotificationChannel::Telegram)
            });
        if rule_exists {
            return Ok(());
        }
        log::debug!("Create non-existent rule {type_code} for user {user_id}.",);
        let telegram_channel_id = self
            .app_postgres
            .get_user_notification_channels(user_id)
            .await?
            .iter()
            .find(|a| a.channel == NotificationChannel::Telegram)
            .unwrap_or_else(|| {
                panic!("User {user_id} does not have a Telegram notification channel.",)
            })
            .id;
        let mut channel_id_set = HashSet::default();
        channel_id_set.insert(telegram_channel_id);
        self.app_postgres
            .save_user_notification_rule(
                user_id,
                &type_code.to_string(),
                (None, None),
                (Some(CONFIG.substrate.network_id), true),
                (&NotificationPeriodType::Off, 0),
                (&HashSet::default(), &channel_id_set, &[]),
            )
            .await?;
        Ok(())
    }

    async fn process_notification_on_off_setting_query(
        &self,
        user_id: u32,
        query: &Query,
        type_code: NotificationTypeCode,
    ) -> anyhow::Result<()> {
        self.create_rule_if_not_exists(user_id, type_code).await?;
        let is_on: bool = serde_json::from_str(query.parameter.as_ref().unwrap_or_else(|| {
            panic!("Expecting on/off param for {type_code} notification setting.",)
        }))?;
        self.app_postgres
            .update_user_notification_rule_period(
                user_id,
                type_code,
                if is_on {
                    NotificationPeriodType::Immediate
                } else {
                    NotificationPeriodType::Off
                },
                0,
            )
            .await?;
        self.app_postgres
            .update_user_pending_notifications_period_by_type(
                user_id,
                type_code,
                if is_on {
                    NotificationPeriodType::Immediate
                } else {
                    NotificationPeriodType::Off
                },
                0,
            )
            .await?;
        Ok(())
    }

    async fn process_notification_period_setting_query(
        &self,
        user_id: u32,
        query: &Query,
        type_code: NotificationTypeCode,
    ) -> anyhow::Result<()> {
        self.create_rule_if_not_exists(user_id, type_code).await?;
        let period_data: (NotificationPeriodType, u16) = serde_json::from_str(
            query.parameter.as_ref().unwrap_or_else(||
                panic!("Expecting period type and period param for block authorship notification setting action.")
            )
        )?;
        self.app_postgres
            .update_user_notification_rule_period(user_id, type_code, period_data.0, period_data.1)
            .await?;
        self.app_postgres
            .update_user_pending_notifications_period_by_type(
                user_id,
                type_code,
                period_data.0,
                period_data.1,
            )
            .await?;
        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    pub async fn process_settings_edit_query(
        &self,
        chat_id: i64,
        query: &Query,
        edit_query_type: &SettingsEditQueryType,
    ) -> anyhow::Result<()> {
        let user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
        let sub_section = match edit_query_type {
            SettingsEditQueryType::Active => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorActive,
                )
                .await?;
                SettingsSubSection::ActiveInactive
            }
            SettingsEditQueryType::ActiveNextSession => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorActiveNextSession,
                )
                .await?;
                SettingsSubSection::ActiveInactive
            }
            SettingsEditQueryType::Inactive => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorInactive,
                )
                .await?;
                SettingsSubSection::ActiveInactive
            }
            SettingsEditQueryType::InactiveNextSession => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorInactiveNextSession,
                )
                .await?;
                SettingsSubSection::ActiveInactive
            }
            SettingsEditQueryType::BlockAuthorship => {
                self.process_notification_period_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorBlockAuthorship,
                )
                .await?;
                SettingsSubSection::BlockAuthorship
            }
            SettingsEditQueryType::Chilled => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorChilled,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::IdentityChanged => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorIdentityChanged,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::OfflineOffence => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorOfflineOffence,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::PayoutStakers => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorPayoutStakers,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::SessionKeysChanged => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorSessionKeysChanged,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::SetController => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorSetController,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::UnclaimedPayout => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorUnclaimedPayout,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::NewNomination => {
                self.process_notification_period_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorNewNomination,
                )
                .await?;
                SettingsSubSection::NewNomination
            }
            SettingsEditQueryType::LostNomination => {
                self.process_notification_period_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorLostNomination,
                )
                .await?;
                SettingsSubSection::LostNomination
            }
            SettingsEditQueryType::StartedParaValidating => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorStartedParaValidating,
                )
                .await?;
                SettingsSubSection::ParaValidation
            }
            SettingsEditQueryType::StoppedParaValidating => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ChainValidatorStoppedParaValidating,
                )
                .await?;
                SettingsSubSection::ParaValidation
            }
            SettingsEditQueryType::OneKVRankChange => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::OneKVValidatorRankChange,
                )
                .await?;
                SettingsSubSection::OneKV
            }
            SettingsEditQueryType::OneKVValidityChange => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::OneKVValidatorValidityChange,
                )
                .await?;
                SettingsSubSection::OneKV
            }
            SettingsEditQueryType::OneKVLocationChange => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::OneKVValidatorLocationChange,
                )
                .await?;
                SettingsSubSection::OneKV
            }
            SettingsEditQueryType::OneKVOnlineStatusChange => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::OneKVValidatorOnlineStatusChange,
                )
                .await?;
                SettingsSubSection::OneKV
            }
            SettingsEditQueryType::ReferendumApproved => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumApproved,
                )
                .await?;
                SettingsSubSection::Referenda
            }
            SettingsEditQueryType::ReferendumCancelled => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumCancelled,
                )
                .await?;
                SettingsSubSection::Referenda
            }
            SettingsEditQueryType::ReferendumConfirmed => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumConfirmed,
                )
                .await?;
                SettingsSubSection::Referenda
            }
            SettingsEditQueryType::ReferendumDecisionStarted => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumDecisionStarted,
                )
                .await?;
                SettingsSubSection::Referenda
            }
            SettingsEditQueryType::ReferendumKilled => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumKilled,
                )
                .await?;
                SettingsSubSection::Referenda
            }
            SettingsEditQueryType::ReferendumRejected => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumRejected,
                )
                .await?;
                SettingsSubSection::Referenda
            }
            SettingsEditQueryType::ReferendumSubmitted => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumSubmitted,
                )
                .await?;
                SettingsSubSection::Referenda
            }
            SettingsEditQueryType::ReferendumTimedOut => {
                self.process_notification_on_off_setting_query(
                    user_id,
                    query,
                    NotificationTypeCode::ReferendumTimedOut,
                )
                .await?;
                SettingsSubSection::Referenda
            }
        };
        let notification_rules = self
            .app_postgres
            .get_user_notification_rules(user_id)
            .await?;
        if let Some(settings_message_id) = self
            .network_postgres
            .get_chat_settings_message_id(chat_id)
            .await?
        {
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

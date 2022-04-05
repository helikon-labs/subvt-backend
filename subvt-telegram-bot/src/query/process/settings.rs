use crate::query::{Query, SettingsEditQueryType, SettingsSubSection};
use crate::TelegramBot;
use subvt_types::app::{NotificationPeriodType, NotificationTypeCode};

impl TelegramBot {
    async fn edit_notification_setting(
        &self,
        user_id: u32,
        query: &Query,
        type_code: &NotificationTypeCode,
    ) -> anyhow::Result<()> {
        let on: bool = serde_json::from_str(query.parameter.as_ref().unwrap_or_else(|| {
            panic!(
                "Expecting on/off param for {} notification setting.",
                type_code
            )
        }))?;
        self.app_postgres
            .update_user_notification_rule_period(
                user_id,
                type_code,
                if on {
                    &NotificationPeriodType::Immediate
                } else {
                    &NotificationPeriodType::Off
                },
                0,
            )
            .await?;
        Ok(())
    }

    pub async fn process_settings_edit_query(
        &self,
        chat_id: i64,
        query: &Query,
        edit_query_type: &SettingsEditQueryType,
    ) -> anyhow::Result<()> {
        let user_id = self.network_postgres.get_chat_app_user_id(chat_id).await?;
        let sub_section = match edit_query_type {
            SettingsEditQueryType::Active => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorActive,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::ActiveNextSession => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorActiveNextSession,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::Inactive => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorInactive,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::InactiveNextSession => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorInactiveNextSession,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::BlockAuthorship => {
                let period_data: (NotificationPeriodType, u16) = serde_json::from_str(
                    query.parameter.as_ref().unwrap_or_else(||
                        panic!("Expecting period type and period param for block authorship notification setting action.")
                    )
                )?;
                self.app_postgres
                    .update_user_notification_rule_period(
                        user_id,
                        &NotificationTypeCode::ChainValidatorBlockAuthorship,
                        &period_data.0,
                        period_data.1,
                    )
                    .await?;
                SettingsSubSection::BlockAuthorship
            }
            SettingsEditQueryType::Chilled => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorChilled,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::IdentityChanged => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorIdentityChanged,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::OfflineOffence => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorOfflineOffence,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::PayoutStakers => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorPayoutStakers,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::SessionKeysChanged => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorSessionKeysChanged,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::SetController => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorSetController,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::UnclaimedPayout => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::ChainValidatorUnclaimedPayout,
                )
                .await?;
                SettingsSubSection::ValidatorActivity
            }
            SettingsEditQueryType::NewNomination => {
                let period_data: (NotificationPeriodType, u16) = serde_json::from_str(
                    query.parameter.as_ref().unwrap_or_else(||
                        panic!("Expecting period type and period param for new nomination notification setting action.")
                    )
                )?;
                self.app_postgres
                    .update_user_notification_rule_period(
                        user_id,
                        &NotificationTypeCode::ChainValidatorNewNomination,
                        &period_data.0,
                        period_data.1,
                    )
                    .await?;
                SettingsSubSection::NewNomination
            }
            SettingsEditQueryType::LostNomination => {
                let period_data: (NotificationPeriodType, u16) = serde_json::from_str(
                    query.parameter.as_ref().unwrap_or_else(||
                        panic!("Expecting period type and period param for lost nomination notification setting action.")
                    )
                )?;
                self.app_postgres
                    .update_user_notification_rule_period(
                        user_id,
                        &NotificationTypeCode::ChainValidatorLostNomination,
                        &period_data.0,
                        period_data.1,
                    )
                    .await?;
                SettingsSubSection::LostNomination
            }
            SettingsEditQueryType::DemocracyCancelled => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyCancelled,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracyDelegated => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyDelegated,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracyNotPassed => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyNotPassed,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracyPassed => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyPassed,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracyProposed => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyProposed,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracySeconded => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracySeconded,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracyStarted => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyStarted,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracyUndelegated => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyUndelegated,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::DemocracyVoted => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::DemocracyVoted,
                )
                .await?;
                SettingsSubSection::Democracy
            }
            SettingsEditQueryType::OneKVRankChange => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::OneKVValidatorRankChange,
                )
                .await?;
                SettingsSubSection::OneKV
            }
            SettingsEditQueryType::OneKVBinaryVersionChange => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::OneKVValidatorBinaryVersionChange,
                )
                .await?;
                SettingsSubSection::OneKV
            }
            SettingsEditQueryType::OneKVValidityChange => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::OneKVValidatorValidityChange,
                )
                .await?;
                SettingsSubSection::OneKV
            }
            SettingsEditQueryType::OneKVLocationChange => {
                self.edit_notification_setting(
                    user_id,
                    query,
                    &NotificationTypeCode::OneKVValidatorLocationChange,
                )
                .await?;
                SettingsSubSection::OneKV
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
                    &sub_section,
                    &notification_rules,
                )
                .await?;
        }
        Ok(())
    }
}

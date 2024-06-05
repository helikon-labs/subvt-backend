use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::notification::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_para_validating(
        &self,
        app_postgres: Arc<PostgreSQLAppStorage>,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.is_para_validator && !last.is_para_validator {
            log::debug!("Started paravalidating: {}", current.account.address);
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorStartedParaValidating.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres,
                &rules,
                &Some(current.account.id),
                None::<&()>,
            )
            .await?;
        } else if !current.is_para_validator && last.is_para_validator {
            log::debug!("Stopped paravalidating: {}", current.account.address);
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorStoppedParaValidating.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres,
                &rules,
                &Some(current.account.id),
                None::<&()>,
            )
            .await?;
        }
        Ok(())
    }
}

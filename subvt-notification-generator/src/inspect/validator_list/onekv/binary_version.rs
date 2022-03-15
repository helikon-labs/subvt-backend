use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::app_event;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_binary_version_change(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.onekv_binary_version != last.onekv_binary_version {
            log::debug!(
                "1KV binary version of {} changed from {:?} to {:?}.",
                current.account.address,
                last.onekv_binary_version,
                current.onekv_binary_version,
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::OneKVValidatorBinaryVersionChange.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres,
                &rules,
                finalized_block_number,
                &current.account.id,
                Some(&app_event::OneKVBinaryVersionChange {
                    validator_account_id: current.account.id.clone(),
                    prev_version: last.onekv_binary_version.clone(),
                    current_version: current.onekv_binary_version.clone(),
                }),
            )
            .await?;
            network_postgres
                .save_onekv_binary_version_change_event(
                    &current.account.id,
                    &last.onekv_binary_version,
                    &current.onekv_binary_version,
                )
                .await?;
        }
        Ok(())
    }
}

use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::app_event;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_online_status_change(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        let (last_offline_since, current_offline_since) =
            match (last.onekv_offline_since, current.onekv_offline_since) {
                (Some(last_offline_since), Some(current_offline_since)) => {
                    (last_offline_since, current_offline_since)
                }
                _ => return Ok(()),
            };
        if (current_offline_since > 0 && last_offline_since == 0)
            || (current_offline_since == 0 && last_offline_since > 0)
        {
            log::debug!(
                "1KV online status (offline_since) of {} changed from {} to {}.",
                current.account.address,
                last_offline_since,
                current_offline_since,
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::OneKVValidatorOnlineStatusChange.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres,
                &rules,
                finalized_block_number,
                &Some(current.account.id),
                Some(&app_event::OneKVOnlineStatusChange {
                    validator_account_id: current.account.id,
                    offline_since: current_offline_since,
                }),
            )
            .await?;
            network_postgres
                .save_onekv_online_status_change_event(&current.account.id, current_offline_since)
                .await?;
        }
        Ok(())
    }
}

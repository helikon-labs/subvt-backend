use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::app::app_event;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_rank_change(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.onekv_rank != last.onekv_rank {
            log::debug!(
                "1KV rank of {} changed from {} to {}.",
                current.account.address,
                last.onekv_rank.unwrap(),
                current.onekv_rank.unwrap(),
            );
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::OneKVValidatorRankChange.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres,
                &rules,
                finalized_block_number,
                &current.account.id,
                Some(&app_event::OneKVRankChange {
                    validator_account_id: current.account.id.clone(),
                    prev_rank: last.onekv_rank.unwrap(),
                    current_rank: current.onekv_rank.unwrap(),
                }),
            )
            .await?;
            network_postgres
                .save_onekv_rank_change_event(
                    &current.account.id,
                    last.onekv_rank.unwrap(),
                    current.onekv_rank.unwrap(),
                )
                .await?;
        }
        Ok(())
    }
}

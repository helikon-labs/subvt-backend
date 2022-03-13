use crate::{NotificationGenerator, CONFIG};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::app::NotificationTypeCode;
use subvt_types::subvt::ValidatorDetails;

impl NotificationGenerator {
    pub(crate) async fn inspect_identity_change(
        &self,
        network_postgres: Arc<PostgreSQLNetworkStorage>,
        app_postgres: Arc<PostgreSQLAppStorage>,
        substrate_client: Arc<SubstrateClient>,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        let parent_identity_changed = if let Some(current_parent) = &*current.account.parent {
            if let Some(last_parent) = &*last.account.parent {
                current_parent.identity != last_parent.identity
            } else {
                current_parent.identity.is_some()
            }
        } else if let Some(last_parent) = &*last.account.parent {
            last_parent.identity.is_some()
        } else {
            false
        };
        if current.account.identity != last.account.identity || parent_identity_changed {
            log::debug!("Identity changed for {}.", current.account.address);
            let rules = app_postgres
                .get_notification_rules_for_validator(
                    &NotificationTypeCode::ChainValidatorIdentityChanged.to_string(),
                    CONFIG.substrate.network_id,
                    &current.account.id,
                )
                .await?;
            self.generate_notifications(
                app_postgres,
                substrate_client,
                &rules,
                finalized_block_number,
                &current.account.id,
                Some(&current.account),
            )
            .await?;
            network_postgres
                .save_identity_changed(
                    &current.account.id,
                    &current.account.get_full_display(),
                    finalized_block_number,
                )
                .await?;
        }
        Ok(())
    }
}

use crate::NotificationGenerator;
use subvt_types::subvt::ValidatorDetails;

mod location;
mod rank;
mod validity;

impl NotificationGenerator {
    pub(crate) async fn inspect_onekv_changes(
        &self,
        finalized_block_number: u64,
        last: &ValidatorDetails,
        current: &ValidatorDetails,
    ) -> anyhow::Result<()> {
        if current.onekv_candidate_record_id.is_none()
            || (current.onekv_candidate_record_id != last.onekv_candidate_record_id)
        {
            return Ok(());
        }
        self.inspect_onekv_rank_change(finalized_block_number, last, current)
            .await?;
        self.inspect_onekv_location_change(finalized_block_number, last, current)
            .await?;
        self.inspect_onekv_validity_change(finalized_block_number, last, current)
            .await?;
        Ok(())
    }
}

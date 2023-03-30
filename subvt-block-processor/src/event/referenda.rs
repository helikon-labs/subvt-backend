use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::event::referenda::ReferendaEvent;

pub(crate) async fn process_referenda_event(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    event_index: usize,
    event: &ReferendaEvent,
) -> anyhow::Result<()> {
    match event {
        ReferendaEvent::Cancelled {
            extrinsic_index,
            referendum_index,
            tally,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_referendum_cancelled_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                    tally.ayes,
                    tally.nays,
                    tally.support,
                )
                .await?;
        }
        ReferendaEvent::Confirmed {
            extrinsic_index,
            referendum_index,
            tally,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_referendum_confirmed_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                    tally.ayes,
                    tally.nays,
                    tally.support,
                )
                .await?;
        }
        ReferendaEvent::DecisionStarted {
            extrinsic_index,
            referendum_index,
            track_id,
            proposal: _,
            tally,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_referendum_decision_started_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                    *track_id,
                    tally.ayes,
                    tally.nays,
                    tally.support,
                )
                .await?;
        }
        ReferendaEvent::Rejected {
            extrinsic_index,
            referendum_index,
            tally,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_referendum_rejected_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                    tally.ayes,
                    tally.nays,
                    tally.support,
                )
                .await?;
        }
        ReferendaEvent::Submitted {
            extrinsic_index,
            referendum_index,
            track_id,
            proposal: _,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_referendum_submitted_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                    *track_id,
                )
                .await?;
        }
    }
    Ok(())
}

pub(crate) async fn update_referenda_event_nesting_index(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    maybe_nesting_index: &Option<String>,
    event_index: i32,
    event: &ReferendaEvent,
) -> anyhow::Result<()> {
    match event {
        ReferendaEvent::Cancelled { .. } => {
            postgres
                .update_referendum_cancelled_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        ReferendaEvent::Confirmed { .. } => {
            postgres
                .update_referendum_confirmed_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        ReferendaEvent::DecisionStarted { .. } => {
            postgres
                .update_referendum_decision_started_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        ReferendaEvent::Rejected { .. } => {
            postgres
                .update_referendum_rejected_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        ReferendaEvent::Submitted { .. } => {
            postgres
                .update_referendum_submitted_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
    }
    Ok(())
}

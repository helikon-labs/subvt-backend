use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::event::DemocracyEvent;

pub(crate) async fn process_democracy_event(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    event_index: usize,
    event: &DemocracyEvent,
) -> anyhow::Result<()> {
    match event {
        DemocracyEvent::Cancelled {
            extrinsic_index,
            referendum_index,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_democracy_cancelled_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                )
                .await?;
        }
        DemocracyEvent::NotPassed {
            extrinsic_index,
            referendum_index,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_democracy_not_passed_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                )
                .await?;
        }
        DemocracyEvent::Passed {
            extrinsic_index,
            referendum_index,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_democracy_passed_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                )
                .await?;
        }
        DemocracyEvent::Started {
            extrinsic_index,
            referendum_index,
            vote_threshold,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_democracy_started_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *referendum_index,
                    &serde_json::to_string(vote_threshold)?,
                )
                .await?;
        }
        _ => (),
    }
    Ok(())
}

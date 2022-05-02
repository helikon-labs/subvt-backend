use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::event::SystemEvent;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_system_event(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    block_number: u64,
    block_timestamp: u64,
    successful_extrinsic_indices: &mut Vec<u32>,
    failed_extrinsic_indices: &mut Vec<u32>,
    event_index: usize,
    event: &SystemEvent,
) -> anyhow::Result<()> {
    match event {
        SystemEvent::ExtrinsicFailed {
            extrinsic_index,
            dispatch_error: _,
            dispatch_info: _,
        } => failed_extrinsic_indices.push(extrinsic_index.unwrap()),
        SystemEvent::ExtrinsicSuccess {
            extrinsic_index,
            dispatch_info: _,
        } => successful_extrinsic_indices.push(extrinsic_index.unwrap()),
        SystemEvent::NewAccount {
            extrinsic_index,
            account_id,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_new_account_event(
                    block_hash,
                    block_number,
                    block_timestamp,
                    extrinsic_index,
                    event_index as i32,
                    account_id,
                )
                .await?;
        }
        SystemEvent::KilledAccount {
            extrinsic_index,
            account_id,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_killed_account_event(
                    block_hash,
                    block_number,
                    block_timestamp,
                    extrinsic_index,
                    event_index as i32,
                    account_id,
                )
                .await?;
        }
        _ => (),
    }
    Ok(())
}

use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::event::SystemEvent;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_system_event(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    block_number: u64,
    block_timestamp: u64,
    event_index: usize,
    event: &SystemEvent,
) -> anyhow::Result<()> {
    match event {
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

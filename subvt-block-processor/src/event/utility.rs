use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::event::UtilityEvent;

pub(crate) async fn process_utility_event(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    event_index: usize,
    event: &UtilityEvent,
) -> anyhow::Result<()> {
    match event {
        UtilityEvent::ItemCompleted { extrinsic_index } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_batch_item_completed_event(block_hash, extrinsic_index, event_index as i32)
                .await?;
        }
        UtilityEvent::BatchInterrupted {
            extrinsic_index,
            item_index,
            dispatch_error,
        } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_batch_interrupted_event(
                    block_hash,
                    extrinsic_index,
                    event_index as i32,
                    *item_index as i32,
                    format!("{:?}", dispatch_error),
                )
                .await?;
        }
        UtilityEvent::BatchCompleted { extrinsic_index } => {
            let extrinsic_index = extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
            postgres
                .save_batch_completed_event(block_hash, extrinsic_index, event_index as i32)
                .await?;
        }
    }
    Ok(())
}

pub(crate) async fn update_utility_event_nesting_index(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    maybe_nesting_index: &Option<String>,
    event_index: i32,
    event: &UtilityEvent,
) -> anyhow::Result<()> {
    match event {
        UtilityEvent::BatchCompleted { .. } => {
            postgres
                .update_batch_completed_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        UtilityEvent::BatchInterrupted { .. } => {
            postgres
                .update_batch_interrupted_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
        UtilityEvent::ItemCompleted { .. } => {
            postgres
                .update_batch_item_completed_event_nesting_index(
                    block_hash,
                    maybe_nesting_index,
                    event_index,
                )
                .await?;
        }
    }
    Ok(())
}

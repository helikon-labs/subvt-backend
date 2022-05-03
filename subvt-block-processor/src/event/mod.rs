use crate::event::democracy::process_democracy_event;
use crate::event::imonline::process_imonline_event;
use crate::event::staking::{process_staking_event, update_staking_event_batch_index};
use crate::event::system::process_system_event;
use crate::event::utility::process_utility_event;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::event::SubstrateEvent;

mod democracy;
mod imonline;
mod staking;
mod system;
mod utility;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_event(
    substrate_client: &SubstrateClient,
    postgres: &PostgreSQLNetworkStorage,
    epoch_index: u64,
    block_hash: &str,
    block_number: u64,
    block_timestamp: u64,
    event_index: usize,
    event: &SubstrateEvent,
) -> anyhow::Result<()> {
    match event {
        SubstrateEvent::Democracy(democracy_event) => {
            process_democracy_event(postgres, block_hash, event_index, democracy_event).await?
        }
        SubstrateEvent::ImOnline(im_online_event) => {
            process_imonline_event(
                substrate_client,
                postgres,
                epoch_index,
                block_hash,
                event_index,
                im_online_event,
            )
            .await?
        }
        SubstrateEvent::Staking(staking_event) => {
            process_staking_event(postgres, block_hash, event_index, staking_event).await?
        }
        SubstrateEvent::System(system_event) => {
            process_system_event(
                postgres,
                block_hash,
                block_number,
                block_timestamp,
                event_index,
                system_event,
            )
            .await?
        }
        SubstrateEvent::Utility(utility_event) => {
            process_utility_event(postgres, block_hash, event_index, utility_event).await?
        }
        _ => (),
    }
    Ok(())
}

pub(crate) async fn update_event_batch_indices(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    batch_index: &Option<String>,
    events: &[(usize, SubstrateEvent)],
) -> anyhow::Result<()> {
    for (event_index, event) in events {
        if let SubstrateEvent::Staking(staking_event) = event {
            update_staking_event_batch_index(
                postgres,
                block_hash,
                batch_index,
                *event_index as i32,
                staking_event,
            )
            .await?;
        }
    }
    Ok(())
}

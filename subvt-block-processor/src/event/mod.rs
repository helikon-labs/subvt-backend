use crate::event::democracy::{process_democracy_event, update_democracy_event_nesting_index};
use crate::event::referenda::{process_referenda_event, update_referenda_event_nesting_index};
use crate::event::staking::{process_staking_event, update_staking_event_nesting_index};
use crate::event::system::{process_system_event, update_system_event_nesting_index};
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::substrate::event::SubstrateEvent;

mod democracy;
mod referenda;
mod staking;
mod system;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_event(
    postgres: &PostgreSQLNetworkStorage,
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
        SubstrateEvent::Referenda(referenda_event) => {
            process_referenda_event(postgres, block_hash, event_index, referenda_event).await?
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
        SubstrateEvent::Utility(_) => (),
        _ => (),
    }
    Ok(())
}

pub(crate) async fn update_event_nesting_indices(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    maybe_nesting_index: &Option<String>,
    events: &[(usize, SubstrateEvent)],
) -> anyhow::Result<()> {
    for (event_index, event) in events {
        match event {
            SubstrateEvent::Democracy(democracy_event) => {
                update_democracy_event_nesting_index(
                    postgres,
                    block_hash,
                    maybe_nesting_index,
                    *event_index as i32,
                    democracy_event,
                )
                .await?;
            }
            SubstrateEvent::Referenda(referenda_event) => {
                update_referenda_event_nesting_index(
                    postgres,
                    block_hash,
                    maybe_nesting_index,
                    *event_index as i32,
                    referenda_event,
                )
                .await?
            }
            SubstrateEvent::Staking(staking_event) => {
                update_staking_event_nesting_index(
                    postgres,
                    block_hash,
                    maybe_nesting_index,
                    *event_index as i32,
                    staking_event,
                )
                .await?;
            }
            SubstrateEvent::System(system_event) => {
                update_system_event_nesting_index(
                    postgres,
                    block_hash,
                    maybe_nesting_index,
                    *event_index as i32,
                    system_event,
                )
                .await?
            }
            SubstrateEvent::Utility(_) => (),
            SubstrateEvent::Identity(_) => {}
            SubstrateEvent::Multisig(_) => {}
            SubstrateEvent::Proxy(_) => {}
            SubstrateEvent::Other { .. } => {}
        }
    }
    Ok(())
}

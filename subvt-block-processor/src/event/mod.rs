use crate::event::imonline::process_imonline_event;
use crate::event::staking::process_staking_event;
use crate::event::system::process_system_event;
use crate::event::utility::process_utility_event;
use crate::BlockProcessor;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::event::SubstrateEvent;

mod imonline;
mod staking;
mod system;
mod utility;

impl BlockProcessor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn process_event(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        epoch_index: u64,
        block_hash: &str,
        block_number: u64,
        block_timestamp: Option<u64>,
        successful_extrinsic_indices: &mut Vec<u32>,
        failed_extrinsic_indices: &mut Vec<u32>,
        event_index: usize,
        event: &SubstrateEvent,
    ) -> anyhow::Result<()> {
        match event {
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
                    successful_extrinsic_indices,
                    failed_extrinsic_indices,
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
}

use crate::extrinsic::imonline::process_imonline_extrinsic;
use crate::extrinsic::staking::process_staking_extrinsic;
use crate::BlockProcessor;
use async_recursion::async_recursion;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::event::{SubstrateEvent, SystemEvent, UtilityEvent};
use subvt_types::substrate::extrinsic::SubstrateExtrinsic;

mod imonline;
mod multisig;
mod proxy;
mod staking;
mod utility;

fn consume_call_events(events: &mut Vec<SubstrateEvent>) -> bool {
    let mut maybe_delimiter_index: Option<usize> = None;
    let mut is_successful = true;
    for (index, event) in events.iter().enumerate() {
        match event {
            SubstrateEvent::System(SystemEvent::ExtrinsicSuccess { .. }) => {
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::System(SystemEvent::ExtrinsicFailed { .. }) => {
                is_successful = false;
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::Utility(UtilityEvent::ItemCompleted { .. }) => {
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::Utility(UtilityEvent::BatchInterrupted { .. }) => {
                is_successful = false;
                maybe_delimiter_index = Some(index);
                break;
            }
            _ => (),
        }
    }
    if let Some(delimiter_index) = maybe_delimiter_index {
        events.drain(0..(delimiter_index + 1));
        is_successful
    } else {
        panic!(
            "Call delimiter event not found (ExtrinsicSuccess, ExtrinsicFailed or ItemCompleted)."
        );
    }
}

impl BlockProcessor {
    #[async_recursion]
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn process_extrinsic(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        block_hash: String,
        block_number: u64,
        active_validator_account_ids: &[AccountId],
        index: usize,
        is_nested_call: bool,
        batch_index: Option<String>,
        maybe_multisig_account_id: Option<AccountId>,
        maybe_real_account_id: Option<AccountId>,
        events: &mut Vec<SubstrateEvent>,
        batch_fail: bool,
        extrinsic: &SubstrateExtrinsic,
    ) -> anyhow::Result<bool> {
        match extrinsic {
            SubstrateExtrinsic::ImOnline(imonline_extrinsic) => {
                let is_successful = !batch_fail && consume_call_events(events);
                process_imonline_extrinsic(
                    postgres,
                    &block_hash,
                    active_validator_account_ids,
                    index,
                    is_nested_call,
                    batch_index,
                    is_successful,
                    imonline_extrinsic,
                )
                .await?;
                Ok(is_successful)
            }
            SubstrateExtrinsic::Multisig(multisig_extrinsic) => {
                let is_successful = self
                    .process_multisig_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        batch_index,
                        events,
                        batch_fail,
                        multisig_extrinsic,
                    )
                    .await?;
                Ok(is_successful)
            }
            SubstrateExtrinsic::Proxy(proxy_extrinsic) => {
                let is_successful = self
                    .process_proxy_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        batch_index,
                        maybe_multisig_account_id,
                        events,
                        batch_fail,
                        proxy_extrinsic,
                    )
                    .await?;
                Ok(is_successful)
            }
            SubstrateExtrinsic::Staking(staking_extrinsic) => {
                let is_successful = !batch_fail && consume_call_events(events);
                process_staking_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    index,
                    is_nested_call,
                    batch_index,
                    maybe_multisig_account_id,
                    maybe_real_account_id,
                    is_successful,
                    staking_extrinsic,
                )
                .await?;
                Ok(is_successful)
            }
            SubstrateExtrinsic::Utility(utility_extrinsic) => {
                let is_successful = self
                    .process_utility_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        batch_index,
                        maybe_multisig_account_id,
                        events,
                        batch_fail,
                        utility_extrinsic,
                    )
                    .await?;
                Ok(is_successful)
            }
            _ => Ok(!batch_fail && consume_call_events(events)),
        }
    }
}

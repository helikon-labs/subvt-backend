use crate::event::update_event_nesting_indices;
use crate::extrinsic::staking::process_staking_extrinsic;
use crate::BlockProcessor;
use async_recursion::async_recursion;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::event::{
    multisig::MultisigEvent, proxy::ProxyEvent, system::SystemEvent, utility::UtilityEvent,
    SubstrateEvent,
};
use subvt_types::substrate::extrinsic::SubstrateExtrinsic;

mod multisig;
mod proxy;
mod staking;
mod utility;

async fn consume_call_events(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    maybe_nesting_index: &Option<String>,
    events: &mut Vec<(usize, SubstrateEvent)>,
) -> anyhow::Result<bool> {
    let mut maybe_delimiter_index: Option<usize> = None;
    let mut is_successful = true;
    for (index, event) in events.iter().enumerate() {
        match event.1 {
            SubstrateEvent::Utility(UtilityEvent::ItemCompleted { .. }) => {
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::Utility(UtilityEvent::ItemFailed { .. }) => {
                is_successful = false;
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::Utility(UtilityEvent::BatchInterrupted { .. }) => {
                is_successful = false;
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::Proxy(ProxyEvent::ProxyExecuted {
                result: dispatch_result,
                ..
            }) => {
                is_successful = dispatch_result.is_ok();
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::Multisig(MultisigEvent::MultisigExecuted {
                result: dispatch_result,
                ..
            }) => {
                is_successful = dispatch_result.is_ok();
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::System(SystemEvent::ExtrinsicFailed { .. }) => {
                is_successful = false;
                maybe_delimiter_index = Some(index);
                break;
            }
            SubstrateEvent::System(SystemEvent::ExtrinsicSuccess { .. }) => {
                maybe_delimiter_index = Some(index);
                break;
            }
            _ => (),
        }
    }
    if let Some(delimiter_index) = maybe_delimiter_index {
        update_event_nesting_indices(
            postgres,
            block_hash,
            maybe_nesting_index,
            &events[0..(delimiter_index + 1)],
        )
        .await?;
        events.drain(0..(delimiter_index + 1));
        Ok(is_successful)
    } else {
        Err(anyhow::anyhow!("Call delimiter event not found (ItemCompleted, ItemFailed, BatchInterrupted, ProxyExecuted, MultisigExecuted, ExtrinsicSuccess, ExtrinsicFailed)."))
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
        maybe_nesting_index: &Option<String>,
        maybe_multisig_account_id: Option<AccountId>,
        maybe_real_account_id: Option<AccountId>,
        events: &mut Vec<(usize, SubstrateEvent)>,
        batch_fail: bool,
        extrinsic: &SubstrateExtrinsic,
    ) -> anyhow::Result<bool> {
        match extrinsic {
            SubstrateExtrinsic::Multisig(multisig_extrinsic) => {
                let is_successful = self
                    .process_multisig_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        maybe_nesting_index,
                        maybe_real_account_id,
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
                        maybe_nesting_index,
                        maybe_multisig_account_id,
                        events,
                        batch_fail,
                        proxy_extrinsic,
                    )
                    .await?;
                Ok(is_successful)
            }
            SubstrateExtrinsic::Staking(staking_extrinsic) => {
                let is_successful = !batch_fail
                    && consume_call_events(postgres, &block_hash, maybe_nesting_index, events)
                        .await?;
                process_staking_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    index,
                    is_nested_call,
                    maybe_nesting_index,
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
                        maybe_nesting_index,
                        maybe_multisig_account_id,
                        maybe_real_account_id,
                        events,
                        batch_fail,
                        utility_extrinsic,
                    )
                    .await?;
                Ok(is_successful)
            }
            _ => Ok(!batch_fail
                && consume_call_events(postgres, &block_hash, maybe_nesting_index, events).await?),
        }
    }
}

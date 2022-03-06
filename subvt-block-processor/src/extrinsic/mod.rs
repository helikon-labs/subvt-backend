use crate::extrinsic::imonline::process_imonline_extrinsic;
use crate::extrinsic::staking::process_staking_extrinsic;
use crate::BlockProcessor;
use async_recursion::async_recursion;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::extrinsic::SubstrateExtrinsic;

mod imonline;
mod multisig;
mod proxy;
mod staking;
mod utility;

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
        maybe_multisig_account_id: Option<AccountId>,
        maybe_real_account_id: Option<AccountId>,
        is_successful: bool,
        extrinsic: &SubstrateExtrinsic,
    ) -> anyhow::Result<()> {
        match extrinsic {
            SubstrateExtrinsic::ImOnline(imonline_extrinsic) => {
                process_imonline_extrinsic(
                    postgres,
                    &block_hash,
                    active_validator_account_ids,
                    index,
                    is_nested_call,
                    is_successful,
                    imonline_extrinsic,
                )
                .await?
            }
            SubstrateExtrinsic::Multisig(multisig_extrinsic) => {
                self.process_multisig_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    block_number,
                    active_validator_account_ids,
                    index,
                    is_successful,
                    multisig_extrinsic,
                )
                .await?
            }
            SubstrateExtrinsic::Proxy(proxy_extrinsic) => {
                self.process_proxy_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    block_number,
                    active_validator_account_ids,
                    index,
                    maybe_multisig_account_id,
                    is_successful,
                    proxy_extrinsic,
                )
                .await?
            }
            SubstrateExtrinsic::Staking(staking_extrinsic) => {
                process_staking_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    index,
                    is_nested_call,
                    maybe_multisig_account_id,
                    maybe_real_account_id,
                    is_successful,
                    staking_extrinsic,
                )
                .await?
            }
            SubstrateExtrinsic::Utility(utility_extrinsic) => {
                self.process_utility_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    block_number,
                    active_validator_account_ids,
                    index,
                    maybe_multisig_account_id,
                    is_successful,
                    utility_extrinsic,
                )
                .await?
            }
            _ => (),
        }
        Ok(())
    }
}

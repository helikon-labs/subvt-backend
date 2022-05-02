use crate::BlockProcessor;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::event::SubstrateEvent;
use subvt_types::substrate::extrinsic::ProxyExtrinsic;

impl BlockProcessor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn process_proxy_extrinsic(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        block_hash: String,
        block_number: u64,
        active_validator_account_ids: &[AccountId],
        index: usize,
        batch_index: Option<String>,
        maybe_multisig_account_id: Option<AccountId>,
        events: &mut Vec<SubstrateEvent>,
        batch_fail: bool,
        extrinsic: &ProxyExtrinsic,
    ) -> anyhow::Result<bool> {
        match extrinsic {
            ProxyExtrinsic::Proxy {
                maybe_signature: _,
                real_account_id,
                force_proxy_type: _,
                call,
            } => {
                let is_successful = self
                    .process_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        true,
                        batch_index,
                        maybe_multisig_account_id,
                        Some(*real_account_id),
                        events,
                        batch_fail,
                        call,
                    )
                    .await?;
                Ok(is_successful)
            }
            ProxyExtrinsic::ProxyAnnounced {
                maybe_signature: _,
                delegate_account_id: _,
                real_account_id,
                force_proxy_type: _,
                call,
            } => {
                let is_successful = self
                    .process_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        true,
                        batch_index,
                        maybe_multisig_account_id,
                        Some(*real_account_id),
                        events,
                        batch_fail,
                        call,
                    )
                    .await?;
                Ok(is_successful)
            }
        }
    }
}

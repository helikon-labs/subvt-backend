use crate::BlockProcessor;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::extrinsic::UtilityExtrinsic;

impl BlockProcessor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn process_utility_extrinsic(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        block_hash: String,
        block_number: u64,
        active_validator_account_ids: &[AccountId],
        index: usize,
        maybe_multisig_account_id: Option<AccountId>,
        is_successful: bool,
        extrinsic: &UtilityExtrinsic,
    ) -> anyhow::Result<()> {
        match extrinsic {
            UtilityExtrinsic::Batch {
                maybe_signature: _,
                calls,
            } => {
                for call in calls {
                    self.process_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash.clone(),
                        block_number,
                        active_validator_account_ids,
                        index,
                        true,
                        maybe_multisig_account_id,
                        None,
                        is_successful,
                        call,
                    )
                    .await?;
                }
            }
            UtilityExtrinsic::BatchAll {
                maybe_signature: _,
                calls,
            } => {
                for call in calls {
                    self.process_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash.clone(),
                        block_number,
                        active_validator_account_ids,
                        index,
                        true,
                        maybe_multisig_account_id,
                        None,
                        is_successful,
                        call,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }
}

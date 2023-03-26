use crate::BlockProcessor;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::event::SubstrateEvent;
use subvt_types::substrate::extrinsic::utility::UtilityExtrinsic;

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
        maybe_nesting_index: &Option<String>,
        maybe_multisig_account_id: Option<AccountId>,
        maybe_real_account_id: Option<AccountId>,
        events: &mut Vec<(usize, SubstrateEvent)>,
        batch_fail: bool,
        extrinsic: &UtilityExtrinsic,
    ) -> anyhow::Result<bool> {
        match extrinsic {
            UtilityExtrinsic::Batch {
                maybe_signature: _,
                calls,
            } => {
                let mut is_successful = !batch_fail;
                for (batch_index, call) in calls.iter().enumerate() {
                    let call_is_successful = self
                        .process_extrinsic(
                            substrate_client,
                            postgres,
                            block_hash.clone(),
                            block_number,
                            active_validator_account_ids,
                            index,
                            true,
                            &if let Some(nesting_index) = maybe_nesting_index.as_ref() {
                                Some(format!("{nesting_index}{batch_index}"))
                            } else {
                                Some(batch_index.to_string())
                            },
                            maybe_multisig_account_id,
                            maybe_real_account_id,
                            events,
                            !is_successful,
                            call,
                        )
                        .await?;
                    is_successful = is_successful && call_is_successful;
                }
                // batch call always returns ok regardless of an interruption
                Ok(true)
            }
            UtilityExtrinsic::BatchAll {
                maybe_signature: _,
                calls,
            } => {
                let mut is_successful = !batch_fail;
                for (batch_index, call) in calls.iter().enumerate() {
                    let call_is_successful = self
                        .process_extrinsic(
                            substrate_client,
                            postgres,
                            block_hash.clone(),
                            block_number,
                            active_validator_account_ids,
                            index,
                            true,
                            &if let Some(nesting_index) = maybe_nesting_index.as_ref() {
                                Some(format!("{nesting_index}{batch_index}"))
                            } else {
                                Some(batch_index.to_string())
                            },
                            maybe_multisig_account_id,
                            maybe_real_account_id,
                            events,
                            !is_successful,
                            call,
                        )
                        .await?;
                    is_successful = is_successful && call_is_successful;
                }
                Ok(is_successful)
            }
            UtilityExtrinsic::ForceBatch {
                maybe_signature: _,
                calls,
            } => {
                for (batch_index, call) in calls.iter().enumerate() {
                    self.process_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash.clone(),
                        block_number,
                        active_validator_account_ids,
                        index,
                        true,
                        &if let Some(nesting_index) = maybe_nesting_index.as_ref() {
                            Some(format!("{nesting_index}{batch_index}"))
                        } else {
                            Some(batch_index.to_string())
                        },
                        maybe_multisig_account_id,
                        maybe_real_account_id,
                        events,
                        false,
                        call,
                    )
                    .await?;
                }
                // force batch call always returns ok regardless of an interruption
                Ok(true)
            }
        }
    }
}

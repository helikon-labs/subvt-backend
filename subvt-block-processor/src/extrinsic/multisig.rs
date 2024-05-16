use crate::BlockProcessor;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::event::SubstrateEvent;
use subvt_types::substrate::extrinsic::multisig::MultisigExtrinsic;

impl BlockProcessor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn process_multisig_extrinsic(
        &self,
        substrate_client: &SubstrateClient,
        postgres: &PostgreSQLNetworkStorage,
        block_hash: String,
        block_number: u64,
        active_validator_account_ids: &[AccountId],
        index: usize,
        maybe_nesting_index: &Option<String>,
        maybe_real_account_id: Option<AccountId>,
        events: &mut Vec<(usize, SubstrateEvent)>,
        batch_fail: bool,
        extrinsic: &MultisigExtrinsic,
    ) -> anyhow::Result<bool> {
        match extrinsic {
            MultisigExtrinsic::AsMulti {
                maybe_signature: signature,
                threshold,
                other_signatories,
                maybe_timepoint: _,
                call,
                ..
            } => {
                let signature = if let Some(signature) = signature {
                    signature
                } else {
                    panic!(
                        "Cannot get signature while processing AsMulti extrinsic {block_number}-{index}.",
                    );
                    /*
                    log::error!(
                        "Cannot get signature while processing AsMulti extrinsic {}-{}.",
                        block_number,
                        index
                    );
                    return Ok(());
                     */
                };
                let multisig_account_id = if let Some(signer_account_id) =
                    signature.get_signer_account_id()
                {
                    AccountId::multisig_account_id(
                        &signer_account_id,
                        other_signatories,
                        *threshold,
                    )
                } else {
                    panic!(
                        "Cannot get multisig account id while processing AsMulti extrinsic {block_number}-{index}.",
                    );
                    /*
                    log::error!(
                        "Cannot get multisig account id while processing AsMulti extrinsic {}-{}.",
                        block_number,
                        index
                    );
                    return Ok(());
                     */
                };
                let is_successful = self
                    .process_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        true,
                        &if let Some(nesting_index) = maybe_nesting_index.as_ref() {
                            Some(format!("{nesting_index}M"))
                        } else {
                            Some("M".to_string())
                        },
                        Some(multisig_account_id),
                        maybe_real_account_id,
                        events,
                        batch_fail,
                        call,
                    )
                    .await?;
                Ok(is_successful)
            }
            MultisigExtrinsic::AsMultiThreshold1 {
                maybe_signature: signature,
                other_signatories,
                call,
            } => {
                let signature = if let Some(signature) = signature {
                    signature
                } else {
                    panic!(
                        "Cannot get signature while processing AsMultiThreshold1 extrinsic {block_number}-{index}.",
                    );
                    /*
                    log::error!(
                        "Cannot get signature while processing AsMultiThreshold1 extrinsic {}-{}.",
                        block_number,
                        index
                    );
                    return Ok(false);
                     */
                };
                let multisig_account_id = if let Some(signer_account_id) =
                    signature.get_signer_account_id()
                {
                    AccountId::multisig_account_id(&signer_account_id, other_signatories, 1)
                } else {
                    panic!("Cannot get multisig account id while processing AsMultiThreshold1 extrinsic {block_number}-{index}.");
                    /*
                    log::error!("Cannot get multisig account id while processing AsMultiThreshold1 extrinsic {}-{}.", block_number, index);
                    return Ok(false);
                     */
                };
                let is_successful = self
                    .process_extrinsic(
                        substrate_client,
                        postgres,
                        block_hash,
                        block_number,
                        active_validator_account_ids,
                        index,
                        true,
                        &if let Some(nesting_index) = maybe_nesting_index.as_ref() {
                            Some(format!("{nesting_index}M"))
                        } else {
                            Some("M".to_string())
                        },
                        Some(multisig_account_id),
                        maybe_real_account_id,
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

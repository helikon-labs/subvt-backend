use crate::BlockProcessor;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::extrinsic::MultisigExtrinsic;

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
        batch_index: Option<String>,
        is_successful: bool,
        extrinsic: &MultisigExtrinsic,
    ) -> anyhow::Result<()> {
        match extrinsic {
            MultisigExtrinsic::AsMulti {
                maybe_signature: signature,
                threshold,
                other_signatories,
                maybe_timepoint: _,
                call,
                store_call: _,
                max_weight: _,
            } => {
                let signature = if let Some(signature) = signature {
                    signature
                } else {
                    log::error!(
                        "Cannot get signature while processing AsMulti extrinsic {}-{}.",
                        block_number,
                        index
                    );
                    return Ok(());
                };
                let multisig_account_id =
                    if let Some(signer_account_id) = signature.get_signer_account_id() {
                        AccountId::multisig_account_id(
                            &signer_account_id,
                            other_signatories,
                            *threshold,
                        )
                    } else {
                        log::error!(
                        "Cannot get multisig account id while processing AsMulti extrinsic {}-{}.",
                        block_number,
                        index
                    );
                        return Ok(());
                    };
                self.process_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    block_number,
                    active_validator_account_ids,
                    index,
                    true,
                    batch_index,
                    Some(multisig_account_id),
                    None,
                    is_successful,
                    &*call,
                )
                .await?;
            }
            MultisigExtrinsic::AsMultiThreshold1 {
                maybe_signature: signature,
                other_signatories,
                call,
            } => {
                let signature = if let Some(signature) = signature {
                    signature
                } else {
                    log::error!(
                        "Cannot get signature while processing AsMultiThreshold1 extrinsic {}-{}.",
                        block_number,
                        index
                    );
                    return Ok(());
                };
                let multisig_account_id = if let Some(signer_account_id) =
                    signature.get_signer_account_id()
                {
                    AccountId::multisig_account_id(&signer_account_id, other_signatories, 1)
                } else {
                    log::error!("Cannot get multisig account id while processing AsMultiThreshold1 extrinsic {}-{}.", block_number, index);
                    return Ok(());
                };
                self.process_extrinsic(
                    substrate_client,
                    postgres,
                    block_hash,
                    block_number,
                    active_validator_account_ids,
                    index,
                    true,
                    batch_index,
                    Some(multisig_account_id),
                    None,
                    is_successful,
                    &*call,
                )
                .await?;
            }
        }
        Ok(())
    }
}

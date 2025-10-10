use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::extrinsic::staking::StakingExtrinsic;
use subvt_types::substrate::MultiAddress;

#[allow(clippy::cognitive_complexity)]
pub(crate) async fn process_staking_extrinsic(
    substrate_client: &SubstrateClient,
    postgres: &PostgreSQLNetworkStorage,
    block_hash: String,
    index: usize,
    is_nested_call: bool,
    maybe_nesting_index: &Option<String>,
    maybe_multisig_account_id: Option<AccountId>,
    maybe_real_account_id: Option<AccountId>,
    is_successful: bool,
    extrinsic: &StakingExtrinsic,
) -> anyhow::Result<()> {
    match extrinsic {
        StakingExtrinsic::Bond { .. } => (),
        StakingExtrinsic::Nominate {
            maybe_signature: signature,
            targets,
        } => {
            let maybe_controller_account_id = if maybe_real_account_id.is_some() {
                maybe_real_account_id
            } else if maybe_multisig_account_id.is_some() {
                maybe_multisig_account_id
            } else {
                match signature {
                    Some(signature) => signature.get_signer_account_id(),
                    _ => None,
                }
            };
            if let Some(controller_account_id) = maybe_controller_account_id {
                let target_account_ids: Vec<AccountId> = targets
                    .iter()
                    .filter_map(|target_multi_address| match target_multi_address {
                        MultiAddress::Id(account_id) => Some(*account_id),
                        _ => {
                            log::error!("Unsupported multi-address type for nomination target.");
                            None
                        }
                    })
                    .collect();
                postgres
                    .save_nominate_extrinsic(
                        &block_hash,
                        index as i32,
                        is_nested_call,
                        maybe_nesting_index,
                        is_successful,
                        &controller_account_id,
                        &target_account_ids,
                    )
                    .await?;
            } else {
                log::error!("Cannot get nominator account id from signature for extrinsic #{index} Staking.nominate.");
            }
        }
        StakingExtrinsic::PayoutStakers {
            maybe_signature: signature,
            validator_account_id,
            era_index,
        } => {
            let maybe_caller_account_id = if maybe_multisig_account_id.is_some() {
                maybe_multisig_account_id
            } else if maybe_real_account_id.is_some() {
                maybe_real_account_id
            } else {
                match signature {
                    Some(signature) => signature.get_signer_account_id(),
                    _ => None,
                }
            };
            if let Some(caller_account_id) = maybe_caller_account_id {
                postgres
                    .save_payout_stakers_extrinsic(
                        &block_hash,
                        index as i32,
                        is_nested_call,
                        maybe_nesting_index,
                        is_successful,
                        &caller_account_id,
                        validator_account_id,
                        *era_index,
                        None,
                    )
                    .await?;
            } else {
                log::error!("Cannot get caller account id from signature for extrinsic #{index} Staking.payout_stakers.");
            }
        }
        StakingExtrinsic::PayoutStakersByPage {
            maybe_signature: signature,
            validator_account_id,
            era_index,
            page_index,
        } => {
            let maybe_caller_account_id = if maybe_multisig_account_id.is_some() {
                maybe_multisig_account_id
            } else if maybe_real_account_id.is_some() {
                maybe_real_account_id
            } else {
                match signature {
                    Some(signature) => signature.get_signer_account_id(),
                    _ => None,
                }
            };
            if let Some(caller_account_id) = maybe_caller_account_id {
                postgres
                    .save_payout_stakers_extrinsic(
                        &block_hash,
                        index as i32,
                        is_nested_call,
                        maybe_nesting_index,
                        is_successful,
                        &caller_account_id,
                        validator_account_id,
                        *era_index,
                        Some(*page_index),
                    )
                    .await?;
            } else {
                log::error!("Cannot get caller account id from signature for extrinsic #{index} Staking.payout_stakers.");
            }
        }
        StakingExtrinsic::Validate {
            maybe_signature: signature,
            preferences,
        } => {
            let maybe_controller_account_id = if maybe_multisig_account_id.is_some() {
                maybe_multisig_account_id
            } else if maybe_real_account_id.is_some() {
                maybe_real_account_id
            } else {
                match signature {
                    Some(signature) => signature.get_signer_account_id(),
                    _ => None,
                }
            };
            if let Some(controller_account_id) = maybe_controller_account_id {
                // get stash account id
                if let Some(stash_account_id) = substrate_client
                    .get_stash_account_id(&controller_account_id, Some(&block_hash))
                    .await?
                {
                    postgres
                        .save_validate_extrinsic(
                            &block_hash,
                            index as i32,
                            is_nested_call,
                            maybe_nesting_index,
                            is_successful,
                            (&stash_account_id, &controller_account_id),
                            preferences,
                        )
                        .await?;
                } else {
                    log::error!(
                        "Cannot get stash account id for controller {controller_account_id}."
                    );
                }
            } else {
                log::error!("Cannot get controller account id from signature for extrinsic #{index} Staking.validate.");
            }
        }
    }
    Ok(())
}

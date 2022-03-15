use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::extrinsic::StakingExtrinsic;
use subvt_types::substrate::MultiAddress;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_staking_extrinsic(
    substrate_client: &SubstrateClient,
    postgres: &PostgreSQLNetworkStorage,
    block_hash: String,
    index: usize,
    is_nested_call: bool,
    maybe_multisig_account_id: Option<AccountId>,
    maybe_real_account_id: Option<AccountId>,
    is_successful: bool,
    extrinsic: &StakingExtrinsic,
) -> anyhow::Result<()> {
    match extrinsic {
        StakingExtrinsic::Bond {
            maybe_signature: signature,
            controller,
            amount,
            reward_destination,
        } => {
            let maybe_stash_account_id = if maybe_real_account_id.is_some() {
                maybe_real_account_id
            } else if maybe_multisig_account_id.is_some() {
                maybe_multisig_account_id
            } else {
                match signature {
                    Some(signature) => signature.get_signer_account_id(),
                    _ => None,
                }
            };
            let controller_account_id = if let Some(account_id) = controller.get_account_id() {
                account_id
            } else {
                log::error!(
                    "Controller address is not raw account id in staking.bond. Cannot persist."
                );
                return Ok(());
            };
            if let Some(stash_account_id) = maybe_stash_account_id {
                postgres
                    .save_bond_extrinsic(
                        &block_hash,
                        index as i32,
                        is_nested_call,
                        is_successful,
                        (
                            &stash_account_id,
                            &controller_account_id,
                            *amount,
                            reward_destination,
                        ),
                    )
                    .await?;
            } else {
                log::error!(
                    "Cannot get caller account id from signature for extrinsic #{} Staking.bond.",
                    index
                );
            }
        }
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
                        MultiAddress::Id(account_id) => Some(account_id.clone()),
                        _ => {
                            log::error!("Unsupported multi-address type for nomination target.");
                            None
                        }
                    })
                    .collect();
                postgres
                    .save_nomination(
                        &block_hash,
                        index as i32,
                        is_nested_call,
                        is_successful,
                        &controller_account_id,
                        &target_account_ids,
                    )
                    .await?;
            } else {
                log::error!("Cannot get nominator account id from signature for extrinsic #{} Staking.nominate.", index);
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
                // ignore the errors here - may fail due to non-existent era foreign key,
                // past eras may not have been saved
                let _ = postgres
                    .save_payout_stakers_extrinsic(
                        &block_hash,
                        index as i32,
                        is_nested_call,
                        is_successful,
                        (&caller_account_id, validator_account_id),
                        *era_index,
                    )
                    .await;
            } else {
                log::error!("Cannot get caller account id from signature for extrinsic #{} Staking.payout_stakers.", index);
            }
        }
        StakingExtrinsic::SetController {
            maybe_signature: signature,
            controller,
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
            let controller_account_id = if let Some(account_id) = controller.get_account_id() {
                account_id
            } else {
                log::error!("Controller address is not raw account id in staking.set_controller. Cannot persist.");
                return Ok(());
            };
            if let Some(caller_account_id) = maybe_caller_account_id {
                postgres
                    .save_set_controller_extrinsic(
                        &block_hash,
                        index as i32,
                        is_nested_call,
                        is_successful,
                        &caller_account_id,
                        &controller_account_id,
                    )
                    .await?;
            } else {
                log::error!("Cannot get caller account id from signature for extrinsic #{} Staking.payout_stakers.", index);
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
                    .get_stash_account_id(&controller_account_id, &block_hash)
                    .await?
                {
                    postgres
                        .save_validate_extrinsic(
                            &block_hash,
                            index as i32,
                            is_nested_call,
                            is_successful,
                            (&stash_account_id, &controller_account_id),
                            preferences,
                        )
                        .await?;
                } else {
                    log::error!(
                        "Cannot get stash account id for controller {}.",
                        controller_account_id.to_string()
                    );
                }
            } else {
                log::error!("Cannot get controller account id from signature for extrinsic #{} Staking.validate.", index);
            }
        }
    }
    Ok(())
}

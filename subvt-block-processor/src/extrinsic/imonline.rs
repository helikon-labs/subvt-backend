use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_types::crypto::AccountId;
use subvt_types::substrate::extrinsic::ImOnlineExtrinsic;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_imonline_extrinsic(
    postgres: &PostgreSQLNetworkStorage,
    block_hash: &str,
    active_validator_account_ids: &[AccountId],
    index: usize,
    is_nested_call: bool,
    batch_index: &Option<String>,
    is_successful: bool,
    extrinsic: &ImOnlineExtrinsic,
) -> anyhow::Result<()> {
    match extrinsic {
        ImOnlineExtrinsic::Hearbeat {
            maybe_signature: _,
            block_number,
            session_index,
            validator_index,
        } => {
            if let Some(validator_account_id) =
                active_validator_account_ids.get(*validator_index as usize)
            {
                let _ = postgres
                    .save_heartbeat_extrinsic(
                        block_hash,
                        index as i32,
                        is_nested_call,
                        batch_index,
                        is_successful,
                        *block_number,
                        *session_index,
                        *validator_index,
                        validator_account_id,
                    )
                    .await?;
            } else {
                log::error!(
                    "Cannot find active validator account id with index {}. Cannot persist heartbeat extrinsic in block {}.",
                    validator_index,
                    block_hash
                );
            }
        }
    }
    Ok(())
}

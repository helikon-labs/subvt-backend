use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;
use subvt_types::substrate::event::im_online::ImOnlineEvent;

pub(crate) async fn process_imonline_event(
    substrate_client: &SubstrateClient,
    postgres: &PostgreSQLNetworkStorage,
    epoch_index: u64,
    block_hash: &str,
    event_index: usize,
    event: &ImOnlineEvent,
) -> anyhow::Result<()> {
    match event {
        ImOnlineEvent::HeartbeatReceived {
            extrinsic_index,
            im_online_key_hex_string,
        } => {
            match substrate_client
                .get_im_online_key_owner_account_id(block_hash, im_online_key_hex_string)
                .await
            {
                Ok(validator_account_id) => {
                    let extrinsic_index =
                        extrinsic_index.map(|extrinsic_index| extrinsic_index as i32);
                    postgres
                        .save_validator_heartbeart_event(
                            block_hash,
                            extrinsic_index,
                            event_index as i32,
                            epoch_index as i64,
                            im_online_key_hex_string,
                            &validator_account_id,
                        )
                        .await?;
                }
                Err(error) => {
                    log::error!("Cannot persist heartbeat event: {error:?}");
                }
            }
        }
        ImOnlineEvent::SomeOffline {
            identification_tuples,
        } => {
            postgres
                .save_validators_offline_event(
                    block_hash,
                    event_index as i32,
                    identification_tuples,
                )
                .await?;
        }
        _ => (),
    }
    Ok(())
}

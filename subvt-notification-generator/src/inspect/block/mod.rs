//! Contains the logic to process new blocks' events and extrinsics and persist notifications
//! to be later sent by `subvt-notification-sender`.

use crate::NotificationGenerator;
use async_lock::Mutex;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_substrate_client::SubstrateClient;

mod authorship;
mod chilling;
mod offence;
mod validate;

impl NotificationGenerator {
    async fn process_block(
        app_postgres: &Arc<PostgreSQLAppStorage>,
        network_postgres: &Arc<PostgreSQLNetworkStorage>,
        substrate_client: &Arc<SubstrateClient>,
        block_number: u64,
    ) -> anyhow::Result<()> {
        log::info!("Process block #{}.", block_number);
        let block = match network_postgres.get_block_by_number(block_number).await? {
            Some(block) => block,
            None => {
                log::error!("Block ${} not found.", block_number);
                return Ok(());
            }
        };
        NotificationGenerator::process_block_authorship(app_postgres, substrate_client, &block)
            .await?;
        NotificationGenerator::process_offline_offences(
            app_postgres,
            network_postgres,
            substrate_client,
            &block,
        )
        .await?;
        NotificationGenerator::process_chillings(
            app_postgres,
            network_postgres,
            substrate_client,
            &block,
        )
        .await?;
        NotificationGenerator::process_validate_extrinsics(
            app_postgres,
            network_postgres,
            substrate_client,
            &block,
        )
        .await?;

        network_postgres
            .save_notification_generator_state(&block.hash, block_number)
            .await
    }

    pub async fn start_processing_blocks(
        config: &'static Config,
        substrate_client: Arc<SubstrateClient>,
    ) -> anyhow::Result<()> {
        let app_postgres =
            Arc::new(PostgreSQLAppStorage::new(config, config.get_app_postgres_url()).await?);
        let network_postgres = Arc::new(
            PostgreSQLNetworkStorage::new(config, config.get_network_postgres_url()).await?,
        );
        let maybe_last_processed_block_number_mutex = Arc::new(Mutex::new(
            network_postgres
                .get_notification_generator_state()
                .await?
                .map(|state| state.1),
        ));
        network_postgres
            .subscribe_to_processed_blocks(|notification| {
                let app_postgres = app_postgres.clone();
                let network_postgres = network_postgres.clone();
                let maybe_last_processed_block_number_mutex =
                    maybe_last_processed_block_number_mutex.clone();
                let substrate_client = substrate_client.clone();
                tokio::spawn(async move {
                    let mut maybe_block_number =
                        maybe_last_processed_block_number_mutex.lock().await;
                    let start_block_number = if let Some(block_number) = *maybe_block_number {
                        block_number + 1
                    } else {
                        notification.block_number
                    };
                    for block_number in start_block_number..=notification.block_number {
                        // process all, update last processed & database
                        match NotificationGenerator::process_block(
                            &app_postgres,
                            &network_postgres,
                            &substrate_client,
                            block_number,
                        )
                        .await
                        {
                            Ok(()) => {
                                *maybe_block_number = Some(block_number);
                            }
                            Err(error) => {
                                log::error!(
                                    "Error while processing block #{}: {:?}",
                                    block_number,
                                    error
                                );
                            }
                        }
                    }
                });
            })
            .await?;
        Ok(())
    }
}

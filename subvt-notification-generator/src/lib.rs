//! Subscribes to the inactive validator list data on Redis and publishes the data
//! through WebSocket pub/sub.

use async_lock::Mutex;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::error;
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct NotificationGenerator;

impl NotificationGenerator {
    async fn process_block(
        &self,
        postgres: &Arc<PostgreSQLNetworkStorage>,
        block_number: u64,
    ) -> anyhow::Result<()> {
        let block = match postgres.get_block_by_number(block_number).await? {
            Some(block) => block,
            None => {
                error!("Block ${} not found.", block_number);
                return Ok(());
            }
        };
        // get authorship notification
        // offences
        // chills
        // commission changes

        postgres
            .save_notification_generator_state(&block.hash, block_number)
            .await
    }
}

#[async_trait(?Send)]
impl Service for NotificationGenerator {
    async fn run(&'static self) -> anyhow::Result<()> {
        let _app_postgres =
            Arc::new(PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?);
        let network_postgres = Arc::new(
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?,
        );
        let maybe_last_processed_block_number_mutex = Arc::new(Mutex::new(
            network_postgres
                .get_notification_generator_state()
                .await?
                .map(|state| state.1),
        ));

        network_postgres
            .subscribe_to_processed_blocks(|notification| {
                let postgres = network_postgres.clone();
                let maybe_last_processed_block_number_mutex =
                    maybe_last_processed_block_number_mutex.clone();
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
                        match self.process_block(&postgres, block_number).await {
                            Ok(()) => {
                                // update database
                                *maybe_block_number = Some(block_number);
                            }
                            Err(error) => {
                                error!(
                                    "Error while processing block #{}: {:?}",
                                    block_number, error
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

//! Contains the logic to process new blocks' events and extrinsics and persist notifications
//! to be later sent by `subvt-notification-sender`.

use crate::NotificationGenerator;
use async_lock::Mutex;
use std::sync::Arc;

mod authorship;
mod chilling;
mod offence;
mod validate;

impl NotificationGenerator {
    async fn inspect_block(&self, block_number: u64) -> anyhow::Result<()> {
        log::info!("Inspect block #{}.", block_number);
        let block = match self
            .network_postgres
            .get_block_by_number(block_number)
            .await?
        {
            Some(block) => block,
            None => {
                log::error!("Block ${} not found.", block_number);
                return Ok(());
            }
        };
        self.inspect_block_authorship(&block).await?;
        self.inspect_offline_offences(&block).await?;
        self.inspect_chillings(&block).await?;
        self.inspect_validate_extrinsics(&block).await?;
        self.network_postgres
            .save_notification_generator_state(&block.hash, block_number)
            .await?;
        log::info!("Completed the inspection of block #{}.", block_number);
        Ok(())
    }

    async fn on_new_block(
        &self,
        last_processed_block_number_mutex: Arc<Mutex<Option<u64>>>,
        new_block_number: u64,
    ) -> anyhow::Result<()> {
        let mut maybe_last_processed_block_number = last_processed_block_number_mutex.lock().await;
        let start_block_number =
            if let Some(last_processed_block_number) = *maybe_last_processed_block_number {
                last_processed_block_number + 1
            } else {
                new_block_number
            };
        for block_number in start_block_number..=new_block_number {
            match self.inspect_block(block_number).await {
                Ok(()) => {
                    *maybe_last_processed_block_number = Some(block_number);
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn start_block_inspection(&'static self) -> anyhow::Result<()> {
        let last_processed_block_number_mutex = Arc::new(Mutex::new(
            self.network_postgres
                .get_notification_generator_state()
                .await?
                .map(|state| state.1),
        ));
        self.network_postgres
            .subscribe_to_processed_blocks(|notification| {
                let last_processed_block_number_mutex = last_processed_block_number_mutex.clone();
                tokio::spawn(async move {
                    if let Err(error) = self
                        .on_new_block(last_processed_block_number_mutex, notification.block_number)
                        .await
                    {
                        log::error!("Error while processing block: {:?}.", error);
                    }
                });
            })
            .await?;
        Ok(())
    }
}

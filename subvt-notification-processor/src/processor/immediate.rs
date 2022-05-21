//! Immediate notification processing logic.
use crate::{NotificationProcessor, CONFIG};
use subvt_types::app::NotificationPeriodType;

impl NotificationProcessor {
    /// Checks and sends notifications that should be sent immediately.
    pub(crate) async fn start_immediate_notification_processor(
        &'static self,
    ) -> anyhow::Result<()> {
        log::info!("Start immediate notification processor.");
        loop {
            log::info!("Check immediate notifications.");
            if let Err(error) = self
                .process_notifications(None, NotificationPeriodType::Immediate, 0)
                .await
            {
                log::error!(
                    "Error while processing immediate notifications: {:?}",
                    error
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(
                CONFIG.notification_processor.sleep_millis,
            ))
            .await;
        }
    }
}

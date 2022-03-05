use crate::{NotificationProcessor, CONFIG};
use log::{error, info};
use subvt_types::app::NotificationPeriodType;

impl NotificationProcessor {
    /// Checks and sends notifications that should be sent immediately.
    pub(crate) async fn start_immediate_notification_processor(&self) -> anyhow::Result<()> {
        info!("Start immediate notification processor.");
        loop {
            info!("Check immediate notifications.");
            if let Err(error) = self
                .process_notifications(NotificationPeriodType::Immediate, 0)
                .await
            {
                error!(
                    "Error while processing immediate notifications: {:?}",
                    error
                );
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(
                CONFIG.notification_processor.sleep_millis,
            ))
            .await;
        }
    }
}

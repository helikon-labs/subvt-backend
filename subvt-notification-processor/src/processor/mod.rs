use crate::NotificationProcessor;
use log::{error, info};
use subvt_types::app::NotificationPeriodType;

pub(crate) mod era_epoch;
pub(crate) mod hour_day;
pub(crate) mod immediate;

impl NotificationProcessor {
    pub(crate) async fn process_notifications(
        &self,
        period_type: NotificationPeriodType,
        period: u32,
    ) -> anyhow::Result<()> {
        info!(
            "Process {} notifications for period {}.",
            period_type, period,
        );
        match self
            .postgres
            .get_pending_notifications_by_period_type(&period_type, period)
            .await
        {
            Ok(notifications) => {
                info!(
                    "Got {} pending {} notifications.",
                    notifications.len(),
                    period_type
                );
                for notification in notifications {
                    let channel = &notification.notification_channel;
                    let sender = match self.senders.get(channel) {
                        Some(sender) => sender.clone(),
                        None => panic!("Sender not found for notification channel: {}", channel),
                    };
                    let postgres = self.postgres.clone();
                    postgres
                        .mark_notification_processing(notification.id)
                        .await?;
                    tokio::spawn(async move {
                        let nofication_id = notification.id;
                        info!(
                            "Send {} notification #{} for {}.",
                            notification.notification_channel,
                            notification.id,
                            notification.validator_account_id.to_ss58_check()
                        );
                        match sender.send(&notification).await {
                            Ok(_success_log) => {
                                let _ = postgres.mark_notification_sent(nofication_id).await;
                            }
                            Err(error) => {
                                error!(
                                    "Error while sending {}-{} notification: {:?}",
                                    period, period_type, error,
                                );
                                let _ = postgres.mark_notification_failed(nofication_id).await;
                            }
                        }
                    });
                }
            }
            Err(error) => error!(
                "Error while getting pending {}-{} notifications: {:?}",
                period, period_type, error
            ),
        }
        Ok(())
    }
}

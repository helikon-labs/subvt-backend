use crate::{metrics, NotificationProcessor};
use subvt_types::app::NotificationPeriodType;

pub(crate) mod era_epoch;
pub(crate) mod hour_day;
pub(crate) mod immediate;

impl NotificationProcessor {
    pub(crate) async fn process_notifications(
        &self,
        maybe_network_id: Option<u32>,
        period_type: NotificationPeriodType,
        period: u32,
    ) -> anyhow::Result<()> {
        log::info!(
            "Process {} notifications for period {}.",
            period_type,
            period,
        );
        match self
            .postgres
            .get_pending_notifications_by_period_type(maybe_network_id, &period_type, period)
            .await
        {
            Ok(notifications) => {
                log::info!(
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
                        log::info!(
                            "Send {} notification #{} for {}.",
                            notification.notification_channel,
                            notification.id,
                            notification.validator_account_id.to_ss58_check()
                        );
                        let start = std::time::Instant::now();
                        match sender.send(&notification).await {
                            Ok(_success_log) => {
                                metrics::sent_notification_counter(&format!(
                                    "{}",
                                    notification.notification_channel
                                ))
                                .inc();
                                metrics::observe_notification_send_time_ms(
                                    &format!("{}", notification.notification_channel),
                                    start.elapsed().as_millis() as f64,
                                );
                                let _ = postgres.mark_notification_sent(nofication_id).await;
                            }
                            Err(error) => {
                                log::error!(
                                    "Error while sending {}-{} notification: {:?}",
                                    period,
                                    period_type,
                                    error,
                                );
                                metrics::channel_error_counter(&format!(
                                    "{}",
                                    notification.notification_channel
                                ))
                                .inc();
                                let _ = postgres.mark_notification_failed(nofication_id).await;
                            }
                        }
                    });
                }
            }
            Err(error) => {
                log::error!(
                    "Error while getting pending {}({}) notifications: {:?}",
                    period,
                    period_type,
                    error
                )
            }
        }
        Ok(())
    }
}

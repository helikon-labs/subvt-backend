//! Contains the notification processing logic.
use crate::{metrics, NotificationProcessor};
use rustc_hash::FxHashMap as HashMap;
use subvt_types::app::notification::{
    Notification, NotificationChannel, NotificationPeriodType, NotificationTypeCode,
};

pub(crate) mod era_epoch;
pub(crate) mod hour_day;
pub(crate) mod immediate;

impl NotificationProcessor {
    async fn process_notification_group(
        &self,
        network_id: u32,
        notification_type_code: &str,
        channel: NotificationChannel,
        target: &str,
        notification_group: Vec<Notification>,
    ) -> anyhow::Result<()> {
        log::debug!(
            "Process {} {} notification group of {} notifications.",
            notification_type_code,
            channel,
            notification_group.len(),
        );
        let sender = self.sender_repository.get_sender(&channel, network_id);
        for notification in notification_group.iter() {
            self.postgres
                .mark_notification_processing(notification.id)
                .await?;
        }
        let postgres = self.postgres.clone();
        let notification_type_code = notification_type_code.to_owned();
        let target = target.to_owned();
        tokio::spawn(async move {
            let start = std::time::Instant::now();
            match sender
                .send_grouped(
                    network_id,
                    &notification_type_code,
                    &channel,
                    &target,
                    &notification_group,
                )
                .await
            {
                Ok(_success_log) => {
                    metrics::sent_notification_counter(&format!("{channel}")).inc();
                    metrics::observe_notification_send_time_ms(
                        &format!("{channel}"),
                        start.elapsed().as_millis() as f64,
                    );
                    for notification in notification_group.iter() {
                        let _ = postgres.mark_notification_sent(notification.id).await;
                    }
                }
                Err(error) => {
                    log::error!("Error while sending grouped notification: {error:?}");
                    metrics::channel_error_counter(&format!("{channel}")).inc();
                    for notification in notification_group.iter() {
                        let _ = postgres.mark_notification_failed(notification.id).await;
                        let _ = postgres
                            .set_notification_error_log(
                                notification.id,
                                format!("{error:?}").as_str(),
                            )
                            .await;
                    }
                }
            }
        });
        Ok(())
    }

    async fn process_single_notification(&self, notification: Notification) -> anyhow::Result<()> {
        log::debug!(
            "Process single {} {} notification.",
            notification.notification_type_code,
            notification.notification_channel,
        );
        let sender = self
            .sender_repository
            .get_sender(&notification.notification_channel, notification.network_id);
        self.postgres
            .mark_notification_processing(notification.id)
            .await?;
        let postgres = self.postgres.clone();
        tokio::spawn(async move {
            let notification_id = notification.id;
            if let Some(account_id) = notification.validator_account_id {
                log::info!(
                    "Send {} {} notification #{} for {}.",
                    notification.notification_type_code,
                    notification.notification_channel,
                    notification.id,
                    account_id.to_ss58_check()
                );
            } else {
                log::info!(
                    "Send {} {} notification #{}.",
                    notification.notification_type_code,
                    notification.notification_channel,
                    notification.id,
                );
            }
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
                    let _ = postgres.mark_notification_sent(notification_id).await;
                }
                Err(error) => {
                    log::error!(
                        "Error while sending {}-{} notification: {:?}",
                        notification.period,
                        notification.period_type,
                        error,
                    );
                    metrics::channel_error_counter(&format!(
                        "{}",
                        notification.notification_channel
                    ))
                    .inc();
                    let _ = postgres.mark_notification_failed(notification_id).await;
                    let _ = postgres
                        .set_notification_error_log(notification_id, format!("{error:?}").as_str())
                        .await;
                }
            }
        });
        Ok(())
    }

    pub(crate) async fn process_notifications(
        &self,
        maybe_network_id: Option<u32>,
        period_type: NotificationPeriodType,
        period: u32,
    ) -> anyhow::Result<()> {
        log::info!("Process {period_type} notifications for period {period}.",);
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
                let mut notification_groups = HashMap::default();
                for notification in &notifications {
                    let key = (
                        notification.network_id,
                        notification.notification_type_code.clone(),
                        notification.validator_account_id,
                        notification.notification_channel,
                        notification.notification_target.clone(),
                    );
                    if !notification_groups.contains_key(&key) {
                        notification_groups.insert(key.clone(), vec![]);
                    }
                    notification_groups
                        .get_mut(&key)
                        .unwrap()
                        .push(notification.clone());
                }
                for (key, notification_group) in notification_groups.into_iter() {
                    if (key.1 == NotificationTypeCode::ChainValidatorBlockAuthorship.to_string()
                        || key.1 == NotificationTypeCode::ChainValidatorNewNomination.to_string()
                        || key.1 == NotificationTypeCode::ChainValidatorLostNomination.to_string())
                        && notification_group.len() > 1
                    {
                        self.process_notification_group(
                            key.0,
                            &key.1,
                            key.3,
                            &key.4,
                            notification_group,
                        )
                        .await?;
                    } else {
                        for notification in notification_group {
                            self.process_single_notification(notification).await?;
                        }
                    }
                }
            }
            Err(error) => {
                log::error!(
                    "Error while getting pending {period}({period_type}) notifications: {error:?}",
                )
            }
        }
        Ok(())
    }
}

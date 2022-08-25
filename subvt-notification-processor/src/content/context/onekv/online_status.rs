use chrono::{TimeZone, Utc};
use subvt_types::app::{app_event, Notification};
use tera::Context;

pub(crate) fn set_onekv_online_status_changed_context(
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) = serde_json::from_str::<app_event::OneKVOnlineStatusChange>(
            notification_data_json.as_str(),
        ) {
            let date_time_format = "%b %d, %Y %H:%M UTC";
            if event.offline_since > 0 {
                let offline_since = Utc::timestamp(&Utc, event.offline_since as i64 / 1000, 0);
                context.insert(
                    "offline_since",
                    &offline_since.format(date_time_format).to_string(),
                );
            }
        } else {
            log::error!(
                "Cannot deserialize 1KV online status changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "1KV online status changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

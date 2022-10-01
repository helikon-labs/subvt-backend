use subvt_types::app::{app_event, notification::Notification};
use tera::Context;

pub(crate) fn set_onekv_rank_changed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<app_event::OneKVRankChange>(notification_data_json.as_str())
        {
            if let Some(prev_rank) = event.prev_rank {
                context.insert("prev_rank", &prev_rank);
            }
            if let Some(current_rank) = event.current_rank {
                context.insert("current_rank", &current_rank);
            }
        } else {
            log::error!(
                "Cannot deserialize 1KV rank changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "1KV rank changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

use subvt_types::app::{app_event, Notification};
use tera::Context;

pub(crate) fn set_onekv_location_changed_context(
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<app_event::OneKVLocationChange>(notification_data_json.as_str())
        {
            if let Some(prev_location) = event.prev_location {
                context.insert("prev_location", &prev_location);
            }
            if let Some(current_location) = event.current_location {
                context.insert("current_location", &current_location);
            }
        } else {
            log::error!(
                "Cannot deserialize 1KV location changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "1KV location changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

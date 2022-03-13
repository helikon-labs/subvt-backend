use subvt_types::app::{app_event, Notification};
use tera::Context;

pub(crate) fn set_onekv_binary_version_changed_context(
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) = serde_json::from_str::<app_event::OneKVBinaryVersionChange>(
            notification_data_json.as_str(),
        ) {
            if let Some(prev_version) = event.prev_version {
                context.insert("prev_version", &prev_version);
            }
            if let Some(current_version) = event.current_version {
                context.insert("current_version", &current_version);
            }
        } else {
            log::error!(
                "Cannot deserialize 1KV binary version changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "1KV binary version changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

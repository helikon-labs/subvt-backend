use subvt_types::app::Notification;
use subvt_utility::text::get_condensed_session_keys;
use tera::Context;

pub(crate) fn set_session_keys_changed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(session_keys) = serde_json::from_str::<String>(notification_data_json.as_str()) {
            context.insert("session_keys", &get_condensed_session_keys(&session_keys));
        } else {
            log::error!(
                "Cannot deserialize session keys changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Session key changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

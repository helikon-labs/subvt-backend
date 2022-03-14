use subvt_types::app::Notification;
use subvt_types::substrate::Account;
use tera::Context;

pub(crate) fn set_identity_changed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(account) = serde_json::from_str::<Account>(notification_data_json.as_str()) {
            if let Some(display) = account.get_full_display() {
                context.insert("identity", &display);
            }
        } else {
            log::error!(
                "Cannot deserialize identity changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Identity changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

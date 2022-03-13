use subvt_types::app::{app_event, Notification};
use tera::Context;

pub(crate) fn set_onekv_validity_changed_context(
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<app_event::OneKVValidityChange>(notification_data_json.as_str())
        {
            context.insert("is_valid", &event.is_valid);
            if event.is_valid {
                let invalidity_reasons: Vec<String> = event
                    .validity_items
                    .iter()
                    .filter(|item| !item.is_valid)
                    .map(|item| item.details.clone())
                    .collect();
                context.insert("invalidity_reasons", &invalidity_reasons);
            }
        } else {
            log::error!(
                "Cannot deserialize 1KV validity changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "1KV validity changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

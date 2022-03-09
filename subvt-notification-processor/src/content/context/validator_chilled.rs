use subvt_types::app::{event, Notification};
use tera::Context;

pub(crate) fn set_validator_chilled_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(chilled_event) =
            serde_json::from_str::<event::ChilledEvent>(notification_data_json.as_str())
        {
            context.insert("block_hash", &chilled_event.block_hash);
            context.insert("event_index", &chilled_event.event_index);
        } else {
            log::error!(
                "Cannot deserialize chilled event notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Chilled event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

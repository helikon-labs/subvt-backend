use subvt_types::app::{event, Notification};
use tera::Context;

pub(crate) fn set_offline_offence_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(offline_event) =
            serde_json::from_str::<event::ValidatorOfflineEvent>(notification_data_json.as_str())
        {
            context.insert("block_hash", &offline_event.block_hash);
            context.insert("event_index", &offline_event.event_index);
        } else {
            log::error!(
                "Cannot deserialize offlince offence event notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Offline offence event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

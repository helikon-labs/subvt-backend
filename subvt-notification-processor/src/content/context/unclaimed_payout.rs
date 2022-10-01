use subvt_types::app::notification::Notification;
use tera::Context;

pub(crate) fn set_unclaimed_payout_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(unclaimed_eras) =
            serde_json::from_str::<Vec<u32>>(notification_data_json.as_str())
        {
            context.insert("unclaimed_eras", &unclaimed_eras);
        } else {
            log::error!(
                "Cannot deserialize unclaimed payouts notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Unclaimed payouts event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

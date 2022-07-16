use subvt_types::app::app_event::CommissionChange;
use subvt_types::app::Notification;
use subvt_utility::numeric::format_decimal;
use tera::Context;

pub(crate) fn set_commission_changed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) = serde_json::from_str::<CommissionChange>(notification_data_json.as_str())
        {
            context.insert(
                "prev_commission_per_cent",
                &format_decimal(event.previous_commission_per_billion as u128, 7, 2),
            );
            context.insert(
                "current_commission_per_billion",
                &format_decimal(event.current_commission_per_billion as u128, 7, 2),
            );
        } else {
            log::error!(
                "Cannot deserialize commission changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Commission changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

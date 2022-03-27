use subvt_types::app::event::democracy::{DemocracyCancelledEvent, DemocracyDelegatedEvent};
use subvt_types::app::{Network, Notification};
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn set_democracy_cancelled_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyCancelledEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize identity changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy cancelled event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_delegated_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyDelegatedEvent>(notification_data_json.as_str())
        {
            let delegate_address = event
                .delegate_account_id
                .to_ss58_check_with_version(network.ss58_prefix as u16);
            context.insert("delegate_address", &delegate_address);
            context.insert(
                "delegate_display",
                &get_condensed_address(&delegate_address, None),
            );
        } else {
            log::error!(
                "Cannot deserialize identity changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy cancelled event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

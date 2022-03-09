use subvt_types::app::{app_event, Network, Notification};
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn set_lost_nomination_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(lost_nomination) =
            serde_json::from_str::<app_event::LostNomination>(notification_data_json.as_str())
        {
            let nominator_address = lost_nomination
                .nominator_stash_account_id
                .to_ss58_check_with_version(network.ss58_prefix as u16);
            context.insert("nominator_address", &nominator_address);
            context.insert(
                "nominator_display",
                &get_condensed_address(&nominator_address, None),
            );
            context.insert(
                "nomination_amount",
                &format_decimal(
                    lost_nomination.active_amount,
                    network.token_decimal_count as usize,
                    4,
                ),
            );
            context.insert("nominee_count", &lost_nomination.nominee_count);
        } else {
            log::error!(
                "Cannot deserialize lost nomination notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Lost nomination data does not exist in notification #{}.",
            notification.id,
        );
    }
}

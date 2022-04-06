use subvt_types::app::{app_event::LostNomination, Network, Notification};
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn set_lost_nomination_grouped_context(
    network: &Network,
    notifications: &[Notification],
    context: &mut Context,
) {
    let mut nomination_count: u32 = 0;
    let mut total_nomination_amount = 0;
    for notification in notifications {
        if let Some(notification_data_json) = &notification.data_json {
            if let Ok(lost_nomination) =
                serde_json::from_str::<LostNomination>(notification_data_json)
            {
                total_nomination_amount += lost_nomination.active_amount;
                nomination_count += 1;
            } else {
                log::error!(
                    "Cannot deserialize lost nomination notification data for grouped notification #{}.",
                    notification.id,
                );
            }
        } else {
            log::error!(
                "Lost nomination data does not exist in grouped notification #{}.",
                notification.id,
            );
        }
    }
    context.insert("nomination_count", &nomination_count);
    context.insert(
        "total_nomination_amount",
        &format_decimal(
            total_nomination_amount,
            network.token_decimal_count as usize,
            4,
        ),
    );
}

pub(crate) fn set_lost_nomination_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(lost_nomination) =
            serde_json::from_str::<LostNomination>(notification_data_json.as_str())
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
            context.insert("is_onekv", &lost_nomination.is_onekv);
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

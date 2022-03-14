use subvt_types::app::{extrinsic, Network, Notification};
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn set_payout_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(extrinsic) = serde_json::from_str::<extrinsic::PayoutStakersExtrinsic>(
            notification_data_json.as_str(),
        ) {
            context.insert("block_hash", &extrinsic.block_hash);
            context.insert("extrinsic_index", &extrinsic.extrinsic_index);
            context.insert("era_index", &extrinsic.era_index);
            let caller_address = extrinsic
                .caller_account_id
                .to_ss58_check_with_version(network.ss58_prefix as u16);
            context.insert("caller_address", &caller_address);
            context.insert(
                "caller_display",
                &get_condensed_address(&caller_address, None),
            );
        } else {
            log::error!(
                "Cannot deserialize payout stakers extrinsic notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Payout stakers extrinsic data does not exist in notification #{}.",
            notification.id,
        );
    }
}

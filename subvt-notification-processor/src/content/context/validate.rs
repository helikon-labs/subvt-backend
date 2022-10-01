use subvt_types::app::{extrinsic, notification::Notification, Network};
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn set_validate_extrinsic_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(extrinsic) =
            serde_json::from_str::<extrinsic::ValidateExtrinsic>(notification_data_json.as_str())
        {
            context.insert("block_hash", &extrinsic.block_hash);
            context.insert("extrinsic_index", &extrinsic.extrinsic_index);
            context.insert("blocks_nominations", &extrinsic.blocks_nominations);
            let controller_address = extrinsic
                .controller_account_id
                .to_ss58_check_with_version(network.ss58_prefix as u16);
            context.insert("controller_address", &controller_address);
            context.insert(
                "controller_display",
                &get_condensed_address(&controller_address, None),
            );
            context.insert(
                "commission",
                &format_decimal(extrinsic.commission_per_billion as u128, 7, 2),
            );
        } else {
            log::error!(
                "Cannot deserialize validate extrinsic notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Validate extrinsic data does not exist in notification #{}.",
            notification.id,
        );
    }
}

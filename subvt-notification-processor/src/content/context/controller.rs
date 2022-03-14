use subvt_types::app::extrinsic::SetControllerExtrinsic;
use subvt_types::app::{Network, Notification};
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn set_controller_changed_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(extrinsic) =
            serde_json::from_str::<SetControllerExtrinsic>(notification_data_json.as_str())
        {
            let controller_address = extrinsic
                .controller_account_id
                .to_ss58_check_with_version(network.ss58_prefix as u16);
            context.insert("controller_address", &controller_address);
            context.insert(
                "controller_display",
                &get_condensed_address(&controller_address, None),
            );
        } else {
            log::error!(
                "Cannot deserialize controller changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Controller changed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

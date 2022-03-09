use subvt_types::app::{Network, Notification};
use subvt_types::substrate::ValidatorStake;
use subvt_utility::numeric::format_decimal;
use tera::Context;

pub(crate) fn set_validator_active_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(validator_stake) =
            serde_json::from_str::<ValidatorStake>(notification_data_json.as_str())
        {
            context.insert("active_nominator_count", &validator_stake.nominators.len());
            context.insert(
                "self_stake",
                &format_decimal(
                    validator_stake.self_stake,
                    network.token_decimal_count as usize,
                    4,
                ),
            );
            context.insert(
                "total_stake",
                &format_decimal(
                    validator_stake.total_stake,
                    network.token_decimal_count as usize,
                    4,
                ),
            );
        } else {
            log::error!(
                "Cannot deserialize validator stake notification data for active notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Validator stake data does not exist in active notification #{}.",
            notification.id,
        );
    }
}

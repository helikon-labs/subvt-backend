use crate::CONFIG;
use subvt_types::app::{Network, Notification, NotificationPeriodType};
use tera::Context;

pub(crate) fn set_basic_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) -> anyhow::Result<()> {
    match notification.period_type {
        NotificationPeriodType::Off | NotificationPeriodType::Immediate => (),
        _ => {
            context.insert(
                "notification_period_type",
                &format!("{}", notification.period_type),
            );
            context.insert("notification_period", &notification.period);
        }
    }
    context.insert("chain", &CONFIG.substrate.chain);
    context.insert(
        "validator_address",
        &notification
            .validator_account_id
            .to_ss58_check_with_version(network.ss58_prefix as u16),
    );
    context.insert(
        "validator_display",
        &if let Some(account) = &notification.get_account()? {
            account.get_display_or_condensed_address(None)
        } else {
            notification
                .validator_account_id
                .to_ss58_check_with_version(network.ss58_prefix as u16)
        },
    );
    context.insert("token_ticker", &network.token_ticker);
    Ok(())
}

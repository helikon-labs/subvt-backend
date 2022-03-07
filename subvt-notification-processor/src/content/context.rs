use crate::CONFIG;
use subvt_types::app::{Block, Network, Notification, NotificationTypeCode};
use subvt_types::substrate::ValidatorStake;
use subvt_utility::numeric::format_decimal;
use tera::Context;

pub(crate) fn get_renderer_context(
    network: &Network,
    notification: &Notification,
) -> anyhow::Result<Context> {
    let mut context = Context::new();
    context.insert("chain", &CONFIG.substrate.chain);
    context.insert(
        "validator_address",
        &notification.validator_account_id.to_ss58_check(),
    );
    context.insert(
        "validator_display",
        &if let Some(account) = &notification.get_account()? {
            account.get_display_or_condensed_address(None)
        } else {
            notification.validator_account_id.to_ss58_check()
        },
    );
    match NotificationTypeCode::from(notification.notification_type_code.as_ref()) {
        NotificationTypeCode::ChainValidatorBlockAuthorship => {
            let block: Block = serde_json::from_str(notification.data_json.as_ref().unwrap())?;
            context.insert("block_number", &block.number);
        }
        NotificationTypeCode::ChainValidatorActive => {
            if let Some(notification_data_json) = &notification.data_json {
                if let Ok(validator_stake) =
                    serde_json::from_str::<ValidatorStake>(notification_data_json.as_str())
                {
                    context.insert("token_ticker", &network.token_ticker);
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
                }
            }
        }
        NotificationTypeCode::ChainValidatorActiveNextSession => (),
        NotificationTypeCode::ChainValidatorInactive => (),
        NotificationTypeCode::ChainValidatorInactiveNextSession => (),
        _ => todo!(
            "Push notification content not yet ready for {}.",
            notification.notification_type_code
        ),
    }
    Ok(context)
}

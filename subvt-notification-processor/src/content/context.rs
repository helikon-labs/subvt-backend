use crate::CONFIG;
use subvt_types::app::{Block, Notification, NotificationTypeCode};
use tera::Context;

pub(crate) fn get_renderer_context(notification: &Notification) -> anyhow::Result<Context> {
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
        NotificationTypeCode::ChainValidatorActive => (),
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

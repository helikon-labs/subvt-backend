use crate::content::context::{
    basic::set_basic_context, block_authorship::set_block_authorship_context,
    lost_nomination::set_lost_nomination_context, new_nomination::set_new_nomination_context,
    offline_offence::set_offline_offence_context, validate::set_validate_extrinsic_context,
    validator_active::set_validator_active_context,
    validator_chilled::set_validator_chilled_context,
};
use subvt_types::app::{Network, Notification, NotificationTypeCode};
use tera::Context;

mod basic;
mod block_authorship;
mod lost_nomination;
mod new_nomination;
mod offline_offence;
mod validate;
mod validator_active;
mod validator_chilled;

pub(crate) fn get_renderer_context(
    network: &Network,
    notification: &Notification,
) -> anyhow::Result<Context> {
    let mut context = Context::new();
    set_basic_context(network, notification, &mut context)?;
    match NotificationTypeCode::from(notification.notification_type_code.as_ref()) {
        NotificationTypeCode::ChainValidatorBlockAuthorship => {
            set_block_authorship_context(notification, &mut context)?;
        }
        NotificationTypeCode::ChainValidatorActive => {
            set_validator_active_context(network, notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorActiveNextSession => (),
        NotificationTypeCode::ChainValidatorInactive => (),
        NotificationTypeCode::ChainValidatorInactiveNextSession => (),
        NotificationTypeCode::ChainValidatorNewNomination => {
            set_new_nomination_context(network, notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorLostNomination => {
            set_lost_nomination_context(network, notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorChilled => {
            set_validator_chilled_context(notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorOfflineOffence => {
            set_offline_offence_context(notification, &mut context);
        }
        NotificationTypeCode::ChainValidateExtrinsic => {
            set_validate_extrinsic_context(network, notification, &mut context);
        }
        _ => todo!(
            "Push notification content not yet ready for {}.",
            notification.notification_type_code
        ),
    }
    Ok(context)
}

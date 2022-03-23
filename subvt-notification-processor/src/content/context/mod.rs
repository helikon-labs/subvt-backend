use crate::content::context::block_authorship::set_block_authorship_grouped_context;
use crate::content::context::{
    basic::set_basic_context,
    block_authorship::set_block_authorship_context,
    controller::set_controller_changed_context,
    identity::set_identity_changed_context,
    lost_nomination::set_lost_nomination_context,
    new_nomination::set_new_nomination_context,
    offline_offence::set_offline_offence_context,
    onekv::{
        binary_version::set_onekv_binary_version_changed_context,
        location::set_onekv_location_changed_context, rank::set_onekv_rank_changed_context,
        validity::set_onekv_validity_changed_context,
    },
    payout::set_payout_context,
    session_keys::set_session_keys_changed_context,
    unclaimed_payout::set_unclaimed_payout_context,
    validate::set_validate_extrinsic_context,
    validator_active::set_validator_active_context,
    validator_chilled::set_validator_chilled_context,
};
use subvt_types::app::{Network, Notification, NotificationTypeCode};
use tera::Context;

mod basic;
mod block_authorship;
mod controller;
mod identity;
mod lost_nomination;
mod new_nomination;
mod offline_offence;
mod onekv;
mod payout;
mod session_keys;
mod unclaimed_payout;
mod validate;
mod validator_active;
mod validator_chilled;

pub(crate) fn get_grouped_renderer_context(
    network: &Network,
    notification_type_code: &str,
    notifications: &[Notification],
) -> anyhow::Result<Context> {
    let mut context = Context::new();
    set_basic_context(network, notifications.get(0).unwrap(), &mut context)?;
    match NotificationTypeCode::from(notification_type_code) {
        NotificationTypeCode::ChainValidatorBlockAuthorship => {
            set_block_authorship_grouped_context(notifications, &mut context)?;
        }
        _ => todo!(
            "Grouped push notification content not yet ready for {}.",
            notification_type_code,
        ),
    }
    Ok(context)
}

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
        NotificationTypeCode::ChainValidatorSessionKeysChanged => {
            set_session_keys_changed_context(notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorSetController => {
            set_controller_changed_context(network, notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorIdentityChanged => {
            set_identity_changed_context(notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorUnclaimedPayout => {
            set_unclaimed_payout_context(notification, &mut context);
        }
        NotificationTypeCode::ChainValidatorPayoutStakers => {
            set_payout_context(network, notification, &mut context);
        }
        NotificationTypeCode::OneKVValidatorBinaryVersionChange => {
            set_onekv_binary_version_changed_context(notification, &mut context);
        }
        NotificationTypeCode::OneKVValidatorRankChange => {
            set_onekv_rank_changed_context(notification, &mut context);
        }
        NotificationTypeCode::OneKVValidatorLocationChange => {
            set_onekv_location_changed_context(notification, &mut context);
        }
        NotificationTypeCode::OneKVValidatorValidityChange => {
            set_onekv_validity_changed_context(notification, &mut context);
        }
        _ => todo!(
            "Push notification content not yet ready for {}.",
            notification.notification_type_code
        ),
    }
    Ok(context)
}

use crate::CONFIG;
use subvt_types::app::{app_event, event, Block, Network, Notification, NotificationTypeCode};
use subvt_types::substrate::ValidatorStake;
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn get_renderer_context(
    network: &Network,
    notification: &Notification,
) -> anyhow::Result<Context> {
    let mut context = Context::new();
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
        NotificationTypeCode::ChainValidatorActiveNextSession => (),
        NotificationTypeCode::ChainValidatorInactive => (),
        NotificationTypeCode::ChainValidatorInactiveNextSession => (),
        NotificationTypeCode::ChainValidatorNewNomination => {
            if let Some(notification_data_json) = &notification.data_json {
                if let Ok(new_nomination) = serde_json::from_str::<app_event::NewNomination>(
                    notification_data_json.as_str(),
                ) {
                    let nominator_address = new_nomination
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
                            new_nomination.active_amount,
                            network.token_decimal_count as usize,
                            4,
                        ),
                    );
                    context.insert("nominee_count", &new_nomination.nominee_count);
                } else {
                    log::error!(
                        "Cannot deserialize new nomination notification data for notification #{}.",
                        notification.id,
                    );
                }
            } else {
                log::error!(
                    "New nomination data does not exist in notification #{}.",
                    notification.id,
                );
            }
        }
        NotificationTypeCode::ChainValidatorLostNomination => {
            if let Some(notification_data_json) = &notification.data_json {
                if let Ok(lost_nomination) = serde_json::from_str::<app_event::LostNomination>(
                    notification_data_json.as_str(),
                ) {
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
        NotificationTypeCode::ChainValidatorChilled => {
            if let Some(notification_data_json) = &notification.data_json {
                if let Ok(chilled_event) =
                    serde_json::from_str::<event::ChilledEvent>(notification_data_json.as_str())
                {
                    context.insert("block_hash", &chilled_event.block_hash);
                    context.insert("event_index", &chilled_event.event_index);
                } else {
                    log::error!(
                        "Cannot deserialize chilled event notification data for notification #{}.",
                        notification.id,
                    );
                }
            } else {
                log::error!(
                    "Chilled event data does not exist in notification #{}.",
                    notification.id,
                );
            }
        }
        NotificationTypeCode::ChainValidatorOfflineOffence => {
            if let Some(notification_data_json) = &notification.data_json {
                if let Ok(offline_event) = serde_json::from_str::<event::ValidatorOfflineEvent>(
                    notification_data_json.as_str(),
                ) {
                    context.insert("block_hash", &offline_event.block_hash);
                    context.insert("event_index", &offline_event.event_index);
                } else {
                    log::error!(
                        "Cannot deserialize offlince offence event notification data for notification #{}.",
                        notification.id,
                    );
                }
            } else {
                log::error!(
                    "Offlince offence event data does not exist in notification #{}.",
                    notification.id,
                );
            }
        }
        _ => todo!(
            "Push notification content not yet ready for {}.",
            notification.notification_type_code
        ),
    }
    Ok(context)
}

use crate::CONFIG;
use subvt_types::app::event::democracy::{
    DemocracyCancelledEvent, DemocracyDelegatedEvent, DemocracyNotPassedEvent,
    DemocracyPassedEvent, DemocracyProposedEvent, DemocracySecondedEvent, DemocracyStartedEvent,
    DemocracyVotedEvent, VoteThreshold,
};
use subvt_types::app::{notification::Notification, Network};
use subvt_utility::numeric::format_decimal;
use subvt_utility::text::get_condensed_address;
use tera::Context;

pub(crate) fn set_democracy_cancelled_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyCancelledEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize democracy cancelled notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy cancelled event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_delegated_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyDelegatedEvent>(notification_data_json.as_str())
        {
            let delegate_address = event
                .delegate_account_id
                .to_ss58_check_with_version(network.ss58_prefix as u16);
            context.insert("delegate_address", &delegate_address);
            context.insert(
                "delegate_display",
                &get_condensed_address(&delegate_address, None),
            );
        } else {
            log::error!(
                "Cannot deserialize democracy delegated notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy delegated event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_not_passed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyNotPassedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize democracy not passed changed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy not passed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_passed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyPassedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize democracy passed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy passed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_proposed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyProposedEvent>(notification_data_json.as_str())
        {
            context.insert("proposal_index", &event.proposal_index);
        } else {
            log::error!(
                "Cannot deserialize democracy proposed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy proposed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_seconded_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracySecondedEvent>(notification_data_json.as_str())
        {
            context.insert("proposal_index", &event.proposal_index);
        } else {
            log::error!(
                "Cannot deserialize democracy seconded notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy seconded event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_started_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyStartedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
            context.insert(
                "vote_threshold",
                match event.vote_threshold {
                    VoteThreshold::SimpleMajority => "SimpleMajority",
                    VoteThreshold::SuperMajorityApprove => "SuperMajorityApprove",
                    VoteThreshold::SuperMajorityAgainst => "SuperMajorityAgainst",
                },
            );
        } else {
            log::error!(
                "Cannot deserialize democracy started notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy started event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_democracy_voted_context(
    network: &Network,
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<DemocracyVotedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
            if let Some(aye_balance) = event.aye_balance {
                let aye_balance_formatted = format_decimal(
                    aye_balance,
                    network.token_decimal_count as usize,
                    CONFIG.substrate.token_format_decimal_points,
                );
                context.insert("aye_balance", &aye_balance_formatted);
            }
            if let Some(nay_balance) = event.nay_balance {
                let nay_balance_formatted = format_decimal(
                    nay_balance,
                    network.token_decimal_count as usize,
                    CONFIG.substrate.token_format_decimal_points,
                );
                context.insert("nay_balance", &nay_balance_formatted);
            }
            if let Some(conviction) = event.conviction {
                context.insert("conviction", &conviction);
            }
        } else {
            log::error!(
                "Cannot deserialize democracy voted notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Democracy voted event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

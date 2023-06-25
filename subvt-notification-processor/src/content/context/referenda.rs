use subvt_types::app::event::referenda::{
    ReferendumApprovedEvent, ReferendumCancelledEvent, ReferendumConfirmedEvent,
    ReferendumDecisionStartedEvent, ReferendumKilledEvent, ReferendumRejectedEvent,
    ReferendumSubmittedEvent, ReferendumTimedOutEvent,
};
use subvt_types::app::notification::Notification;
use tera::Context;

pub(crate) fn set_referendum_approved_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumApprovedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum approved notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum approved event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_referendum_cancelled_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumCancelledEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum cancelled notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum cancelled event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_referendum_confirmed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumConfirmedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum confirmed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum confirmed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_referendum_decision_started_context(
    notification: &Notification,
    context: &mut Context,
) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumDecisionStartedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum decision started notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum decision started event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_referendum_killed_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumKilledEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum killed notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum killed event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_referendum_rejected_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumRejectedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum rejected notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum rejected event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_referendum_submitted_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumSubmittedEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum submitted notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum submitted event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

pub(crate) fn set_referendum_timed_out_context(notification: &Notification, context: &mut Context) {
    if let Some(notification_data_json) = &notification.data_json {
        if let Ok(event) =
            serde_json::from_str::<ReferendumTimedOutEvent>(notification_data_json.as_str())
        {
            context.insert("referendum_index", &event.referendum_index);
        } else {
            log::error!(
                "Cannot deserialize referendum timed out notification data for notification #{}.",
                notification.id,
            );
        }
    } else {
        log::error!(
            "Referendum timed out event data does not exist in notification #{}.",
            notification.id,
        );
    }
}

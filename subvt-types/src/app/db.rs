use crate::app::{
    Network, NotificationParamDataType, NotificationPeriodType, UserNotificationChannel,
    UserValidator,
};
use crate::crypto::AccountId;
use std::str::FromStr;

pub type PostgresNetwork = (
    i32,
    String,
    String,
    i32,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl From<PostgresNetwork> for Network {
    fn from(db_network: PostgresNetwork) -> Self {
        Network {
            id: db_network.0 as u32,
            hash: db_network.1.clone(),
            name: db_network.2.clone(),
            ss58_prefix: db_network.3 as u32,
            live_network_status_service_url: db_network.4.clone(),
            report_service_url: db_network.5.clone(),
            validator_details_service_url: db_network.6.clone(),
            validator_list_service_url: db_network.7,
        }
    }
}

pub type PostgresUserValidator = (i32, i32, i32, String);

impl From<PostgresUserValidator> for UserValidator {
    fn from(db_user_validator: PostgresUserValidator) -> Self {
        UserValidator {
            id: db_user_validator.0 as u32,
            user_id: db_user_validator.1 as u32,
            network_id: db_user_validator.2 as u32,
            validator_account_id: AccountId::from_str(&db_user_validator.3).unwrap(),
        }
    }
}

pub type PostgresUserNotificationChannel = (i32, i32, String, String);

impl From<PostgresUserNotificationChannel> for UserNotificationChannel {
    fn from(db_user_notification_channel: PostgresUserNotificationChannel) -> Self {
        UserNotificationChannel {
            id: db_user_notification_channel.0 as u32,
            user_id: db_user_notification_channel.1 as u32,
            channel_code: db_user_notification_channel.2.clone(),
            target: db_user_notification_channel.3,
        }
    }
}

pub type PostgresUserNotificationRule = (
    i32,
    i32,
    String,
    Option<String>,
    Option<i32>,
    bool,
    NotificationPeriodType,
    i32,
    Option<String>,
);

pub type PostgresNotificationParamType = (
    i32,
    String,
    String,
    i16,
    NotificationParamDataType,
    Option<String>,
    Option<String>,
    bool,
);

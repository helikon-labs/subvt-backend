//! Helper types to read data from PostgreSQL using SQLx.
use crate::app::extrinsic::{PayoutStakersExtrinsic, SetControllerExtrinsic, ValidateExtrinsic};
use crate::app::{
    Block, Network, Notification, NotificationParamDataType, NotificationPeriodType,
    UserNotificationChannel, UserValidator,
};
use crate::crypto::AccountId;
use std::str::FromStr;

pub type PostgresNetwork = (
    i32,
    String,
    String,
    String,
    i32,
    String,
    i32,
    Option<String>,
    Option<String>,
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
            chain: db_network.2.clone(),
            display: db_network.3.clone(),
            ss58_prefix: db_network.4 as u32,
            token_ticker: db_network.5.clone(),
            token_decimal_count: db_network.6 as u8,
            redis_url: db_network.7,
            network_status_service_url: db_network.8,
            report_service_url: db_network.9,
            validator_details_service_url: db_network.10,
            active_validator_list_service_url: db_network.11,
            inactive_validator_list_service_url: db_network.12,
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
            channel: db_user_notification_channel.2.clone().as_str().into(),
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

pub type PostgresBlock = (
    String,
    i64,
    Option<i64>,
    Option<String>,
    i64,
    i64,
    bool,
    i16,
    i16,
);

impl Block {
    pub fn from(db_block: PostgresBlock) -> anyhow::Result<Block> {
        let author_account_id = if let Some(hex_string) = db_block.3 {
            Some(AccountId::from_str(&hex_string)?)
        } else {
            None
        };
        Ok(Block {
            hash: db_block.0,
            number: db_block.1 as u64,
            timestamp: db_block.2.map(|timestamp| timestamp as u64),
            author_account_id,
            era_index: db_block.4 as u64,
            epoch_index: db_block.5 as u64,
            is_finalized: db_block.6,
            metadata_version: db_block.7 as u16,
            runtime_version: db_block.8 as u16,
        })
    }
}

pub type PostgresValidateExtrinsic = (i32, String, i32, bool, String, String, i64, bool, bool);

impl ValidateExtrinsic {
    pub fn from(db_extrinsic: PostgresValidateExtrinsic) -> anyhow::Result<ValidateExtrinsic> {
        Ok(ValidateExtrinsic {
            id: db_extrinsic.0 as u32,
            block_hash: db_extrinsic.1.clone(),
            extrinsic_index: db_extrinsic.2 as u32,
            is_nested_call: db_extrinsic.3,
            stash_account_id: AccountId::from_str(&db_extrinsic.4)?,
            controller_account_id: AccountId::from_str(&db_extrinsic.5)?,
            commission_per_billion: db_extrinsic.6 as u64,
            blocks_nominations: db_extrinsic.7,
            is_successful: db_extrinsic.8,
        })
    }
}

pub type PostgresSetControllerExtrinsic = (i32, String, i32, bool, String, String, bool);

impl SetControllerExtrinsic {
    pub fn from(
        db_extrinsic: PostgresSetControllerExtrinsic,
    ) -> anyhow::Result<SetControllerExtrinsic> {
        Ok(SetControllerExtrinsic {
            id: db_extrinsic.0 as u32,
            block_hash: db_extrinsic.1.clone(),
            extrinsic_index: db_extrinsic.2 as u32,
            is_nested_call: db_extrinsic.3,
            caller_account_id: AccountId::from_str(&db_extrinsic.4)?,
            controller_account_id: AccountId::from_str(&db_extrinsic.5)?,
            is_successful: db_extrinsic.6,
        })
    }
}

pub type PostgresPayoutStakersExtrinsic = (i32, String, i32, bool, String, String, i64, bool);

impl PayoutStakersExtrinsic {
    pub fn from(
        db_extrinsic: PostgresPayoutStakersExtrinsic,
    ) -> anyhow::Result<PayoutStakersExtrinsic> {
        Ok(PayoutStakersExtrinsic {
            id: db_extrinsic.0 as u32,
            block_hash: db_extrinsic.1.clone(),
            extrinsic_index: db_extrinsic.2 as u32,
            is_nested_call: db_extrinsic.3,
            caller_account_id: AccountId::from_str(&db_extrinsic.4)?,
            validator_account_id: AccountId::from_str(&db_extrinsic.5)?,
            era_index: db_extrinsic.6 as u32,
            is_successful: db_extrinsic.7,
        })
    }
}

pub type PostgresNotification = (
    i32,
    i32,
    i32,
    i32,
    NotificationPeriodType,
    i32,
    Option<String>,
    Option<String>,
    String,
    i32,
    String,
    String,
    Option<String>,
    Option<String>,
);

impl Notification {
    pub fn from(db_notification: PostgresNotification) -> anyhow::Result<Notification> {
        Ok(Notification {
            id: db_notification.0 as u32,
            user_id: db_notification.1 as u32,
            user_notification_rule_id: db_notification.2 as u32,
            network_id: db_notification.3 as u32,
            period_type: db_notification.4,
            period: db_notification.5 as u16,
            validator_account_id: if let Some(hex_string) = db_notification.6.as_ref() {
                Some(AccountId::from_str(hex_string)?)
            } else {
                None
            },
            validator_account_json: db_notification.7.clone(),
            notification_type_code: db_notification.8.clone(),
            user_notification_channel_id: db_notification.9 as u32,
            notification_channel: db_notification.10.as_str().into(),
            notification_target: db_notification.11.clone(),
            data_json: db_notification.12.clone(),
            log: db_notification.13.clone(),
            created_at: None,
            sent_at: None,
            delivered_at: None,
            read_at: None,
        })
    }
}

//! Apple Push Notification Service (APNS) sender. Sends notifications to Apple devices.
use crate::sender::{NotificationSender, NotificationSenderError};
use crate::{ContentProvider, CONFIG};
use a2::ErrorReason;
use async_trait::async_trait;
use serde::Serialize;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::notification::{Notification, NotificationChannel};
use subvt_types::crypto::AccountId;
use subvt_types::substrate::Account;

pub(crate) struct APNSSender {
    apns_client: a2::Client,
    content_provider: ContentProvider,
    app_postgres: PostgreSQLAppStorage,
}

impl APNSSender {
    pub async fn new(content_provider: ContentProvider) -> anyhow::Result<APNSSender> {
        let mut apns_key = std::fs::File::open(&CONFIG.notification_processor.apns_key_path)?;
        let apns_client = a2::Client::token(
            &mut apns_key,
            &CONFIG.notification_processor.apns_key_id,
            &CONFIG.notification_processor.apns_team_id,
            if CONFIG.notification_processor.apns_is_production {
                a2::Endpoint::Production
            } else {
                a2::Endpoint::Sandbox
            },
        )?;
        let app_postgres =
            PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?;
        Ok(APNSSender {
            apns_client,
            content_provider,
            app_postgres,
        })
    }
}

#[derive(Serialize, Debug)]
struct APNSNotificationData {
    network_id: u32,
    notification_type_code: String,
    validator_account_id: Option<String>,
    validator_display: Option<String>,
}

impl APNSSender {
    #[allow(clippy::too_many_arguments)]
    async fn send_inner(
        &self,
        network_id: u32,
        notification_type_code: &str,
        maybe_validator_account_id: &Option<AccountId>,
        maybe_validator_account: &Option<Account>,
        message: &str,
        user_notification_channel_id: u32,
        target: &str,
    ) -> anyhow::Result<String> {
        let mut payload = a2::request::payload::Payload {
            options: a2::NotificationOptions {
                apns_topic: Some("io.helikon.subvt"),
                ..Default::default()
            },
            device_token: target,
            aps: a2::request::payload::APS {
                alert: Some(a2::request::payload::APSAlert::Plain(message)),
                badge: None,
                sound: Some("default"),
                content_available: Some(1),
                category: None,
                mutable_content: None,
                url_args: None,
            },
            data: Default::default(),
        };
        payload.add_custom_data(
            "notification_data",
            &APNSNotificationData {
                network_id,
                notification_type_code: notification_type_code.to_string(),
                validator_account_id: maybe_validator_account_id
                    .map(|account_id| account_id.to_string()),
                validator_display: if let Some(account) = maybe_validator_account {
                    account.get_full_display()
                } else {
                    None
                },
            },
        )?;
        match self.apns_client.send(payload).await {
            Ok(response) => {
                log::info!("APNS notification sent succesfully.");
                Ok(format!("{response:?}"))
            }
            Err(error) => {
                log::error!("APNS notification send error: {:?}.", error);
                if let a2::Error::ResponseError(response) = &error {
                    if let Some(error) = &response.error {
                        match error.reason {
                            ErrorReason::BadDeviceToken => {
                                log::error!(
                                    "APNS Error: bad device token. Delete user notification APNS channel #{}.",
                                    user_notification_channel_id
                                );
                                self.app_postgres
                                    .delete_user_notification_channel(user_notification_channel_id)
                                    .await?;
                            }
                            ErrorReason::DeviceTokenNotForTopic => {
                                log::error!(
                                    "APNS Error: device token not for topic. Delete user notification APNS channel #{}.",
                                    user_notification_channel_id
                                );
                                self.app_postgres
                                    .delete_user_notification_channel(user_notification_channel_id)
                                    .await?;
                            }
                            ErrorReason::Unregistered => {
                                log::error!(
                                    "APNS Error: unregistered device token. Delete user notification APNS channel #{}.",
                                    user_notification_channel_id
                                );
                                self.app_postgres
                                    .delete_user_notification_channel(user_notification_channel_id)
                                    .await?;
                            }
                            _ => (),
                        }
                    } else if response.code == 410 {
                        log::warn!(
                            "APNS Error: no response error body. Response code 410. Delete user notification APNS channel #{}.",
                            user_notification_channel_id
                        );
                    }
                }
                Err(NotificationSenderError::Error(format!("{error:?}")).into())
            }
        }
    }
}

#[async_trait]
impl NotificationSender for APNSSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let message = self
            .content_provider
            .get_notification_content(notification)?
            .body_text
            .unwrap_or_else(|| {
                panic!(
                    "Cannot get text content for APNS {} notification.",
                    notification.notification_type_code
                )
            });
        let account = if let Some(json) = &notification.validator_account_json {
            serde_json::from_str::<Account>(json).ok()
        } else {
            None
        };
        self.send_inner(
            notification.network_id,
            &notification.notification_type_code,
            &notification.validator_account_id,
            &account,
            &message,
            notification.user_notification_channel_id,
            &notification.notification_target,
        )
        .await
    }

    async fn send_grouped(
        &self,
        network_id: u32,
        notification_type_code: &str,
        channel: &NotificationChannel,
        target: &str,
        notifications: &[Notification],
    ) -> anyhow::Result<String> {
        let message = self
            .content_provider
            .get_grouped_notification_content(
                network_id,
                notification_type_code,
                channel,
                notifications,
            )?
            .body_text
            .unwrap_or_else(|| {
                panic!("Cannot get text content for APNS {notification_type_code} notification.",)
            });
        let (account_id, account) = if let Some(notification) = notifications.first() {
            let account = if let Some(json) = &notification.validator_account_json {
                serde_json::from_str::<Account>(json).ok()
            } else {
                None
            };
            (notification.validator_account_id, account)
        } else {
            (None, None)
        };
        self.send_inner(
            network_id,
            notification_type_code,
            &account_id,
            &account,
            &message,
            notifications
                .first()
                .map(|notification| notification.user_notification_channel_id)
                .unwrap_or(0),
            target,
        )
        .await
    }
}

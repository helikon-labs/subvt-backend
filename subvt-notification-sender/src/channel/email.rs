use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use log::{debug, error};
use std::sync::Arc;
use subvt_config::Config;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::Notification;

pub(crate) struct EmailSender {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailSender {
    pub(crate) fn new(config: &Config) -> anyhow::Result<EmailSender> {
        Ok(EmailSender {
            mailer: AsyncSmtpTransport::<Tokio1Executor>::relay(
                &config.notification_sender.email_smtp_server_url,
            )?
            .credentials(Credentials::new(
                config.notification_sender.email_account.clone(),
                config.notification_sender.email_password.clone(),
            ))
            .port(config.notification_sender.email_smtp_server_tls_port)
            .build(),
        })
    }
}

pub(crate) async fn send_email(
    config: &Config,
    postgres: &Arc<PostgreSQLAppStorage>,
    email_sender: &Arc<EmailSender>,
    notification: &Notification,
) -> anyhow::Result<()> {
    let message = lettre::Message::builder()
        .from(config.notification_sender.email_from.parse()?)
        .reply_to(config.notification_sender.email_reply_to.parse()?)
        .to("kutsalbilgin@gmail.com".parse()?)
        .subject(format!(
            "Block xyz authored by {}",
            notification.validator_account_id.to_ss58_check()
        ))
        .body(String::from("Your validator has authored a block."))
        .unwrap();
    postgres
        .mark_notification_processing(notification.id)
        .await?;
    match email_sender.mailer.send(message).await {
        Ok(response) => {
            debug!(
                "Mail sent succesfully for notification #{}.",
                notification.id
            );
            postgres.mark_notification_sent(notification.id).await?;
            postgres
                .mark_notification_delivered(notification.id)
                .await?;
            postgres
                .set_notification_log(notification.id, format!("{:?}", response).as_ref())
                .await?;
        }
        Err(error) => {
            error!(
                "Mail send error for notification #{}: {:?}.",
                notification.id, error,
            );
            postgres.mark_notification_failed(notification.id).await?;
            postgres
                .set_notification_log(notification.id, format!("{:?}", error).as_ref())
                .await?;
        }
    }
    Ok(())
}

//! Email sending logic.

use crate::{ContentProvider, CONFIG};
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use log::{debug, error};
use std::sync::Arc;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_types::app::Notification;

pub(crate) type Mailer = AsyncSmtpTransport<Tokio1Executor>;

pub(crate) fn new_mailer() -> anyhow::Result<Mailer> {
    Ok(
        Mailer::relay(&CONFIG.notification_sender.email_smtp_server_url)?
            .credentials(Credentials::new(
                CONFIG.notification_sender.email_account.clone(),
                CONFIG.notification_sender.email_password.clone(),
            ))
            // .port(config.notification_sender.email_smtp_server_tls_port)
            .build(),
    )
}

pub(crate) async fn send_email(
    postgres: &Arc<PostgreSQLAppStorage>,
    mailer: &Arc<Mailer>,
    content_provider: &Arc<ContentProvider>,
    notification: &Notification,
) -> anyhow::Result<()> {
    let (subject, text_body, html_body) = content_provider.get_email_content(notification)?;
    let message = lettre::Message::builder()
        .from(CONFIG.notification_sender.email_from.parse()?)
        .reply_to(CONFIG.notification_sender.email_reply_to.parse()?)
        .to("kutsalbilgin@gmail.com".parse()?)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(text_body),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        )?;
    postgres
        .mark_notification_processing(notification.id)
        .await?;
    match mailer.send(message).await {
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

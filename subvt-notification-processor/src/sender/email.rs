use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender, CONFIG};
use async_trait::async_trait;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use log::{error, info};
use subvt_types::app::Notification;

pub(crate) type Mailer = AsyncSmtpTransport<Tokio1Executor>;

pub(crate) struct EmailSender {
    mailer: Mailer,
    content_provider: ContentProvider,
}

impl EmailSender {
    pub async fn new() -> anyhow::Result<EmailSender> {
        let mailer = Mailer::relay(&CONFIG.notification_processor.email_smtp_server_url)?
            .credentials(Credentials::new(
                CONFIG.notification_processor.email_account.clone(),
                CONFIG.notification_processor.email_password.clone(),
            ))
            // .port(config.notification_sender.email_smtp_server_tls_port)
            .build();
        let content_provider = ContentProvider::new().await?;
        Ok(EmailSender {
            mailer,
            content_provider,
        })
    }
}

#[async_trait]
impl NotificationSender for EmailSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let content = self
            .content_provider
            .get_notification_content(notification)?;
        let (subject, body_text, body_html) = (
            content.subject.unwrap_or_else(|| {
                panic!(
                    "Cannot get subject for email {} notification.",
                    notification.notification_type_code
                )
            }),
            content.body_text.unwrap_or_else(|| {
                panic!(
                    "Cannot get body text for email {} notification.",
                    notification.notification_type_code
                )
            }),
            content.body_html.unwrap_or_else(|| {
                panic!(
                    "Cannot get body html for email {} notification.",
                    notification.notification_type_code
                )
            }),
        );
        let message = lettre::Message::builder()
            .from(CONFIG.notification_processor.email_from.parse()?)
            .reply_to(CONFIG.notification_processor.email_reply_to.parse()?)
            .to("kutsalbilgin@gmail.com".parse()?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(body_text),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(body_html),
                    ),
            )?;
        match self.mailer.send(message).await {
            Ok(response) => {
                info!(
                    "Mail sent succesfully for notification #{}.",
                    notification.id
                );
                Ok(format!("{:?}", response))
            }
            Err(error) => {
                error!(
                    "Mail send error for notification #{}: {:?}.",
                    notification.id, error,
                );
                Err(NotificationSenderError::Error(format!("{:?}", error)).into())
            }
        }
    }
}

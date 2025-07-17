//! Email sender.
use crate::content::NotificationContent;
use crate::sender::NotificationSenderError;
use crate::{ContentProvider, NotificationSender, CONFIG};
use async_trait::async_trait;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use subvt_types::app::notification::{Notification, NotificationChannel};

pub(crate) type Mailer = AsyncSmtpTransport<Tokio1Executor>;

pub(crate) struct EmailSender {
    mailer: Mailer,
    content_provider: ContentProvider,
}

impl EmailSender {
    pub async fn new(content_provider: ContentProvider) -> anyhow::Result<EmailSender> {
        let mailer = Mailer::relay(&CONFIG.notification_processor.email_smtp_server_url)?
            .credentials(Credentials::new(
                CONFIG.notification_processor.email_account.clone(),
                CONFIG.notification_processor.email_password.clone(),
            ))
            // .port(config.notification_sender.email_smtp_server_tls_port)
            .build();
        Ok(EmailSender {
            mailer,
            content_provider,
        })
    }
}

impl EmailSender {
    async fn send_inner(
        &self,
        target: &str,
        content: NotificationContent,
    ) -> anyhow::Result<String> {
        let (subject, body_text, body_html) = (
            content
                .subject
                .unwrap_or_else(|| panic!("Cannot get subject for email notification.")),
            content
                .body_text
                .unwrap_or_else(|| panic!("Cannot get body text for email notification.")),
            content
                .body_html
                .unwrap_or_else(|| panic!("Cannot get body html for email notification.")),
        );
        let message = lettre::Message::builder()
            .from(CONFIG.notification_processor.email_from.parse()?)
            .reply_to(CONFIG.notification_processor.email_reply_to.parse()?)
            .to(target.parse()?)
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
                log::info!("Mail sent succesfully for notification.");
                Ok(format!("{response:?}"))
            }
            Err(error) => {
                log::error!("Mail send error: {error:?}.",);
                Err(NotificationSenderError::Error(format!("{error:?}")).into())
            }
        }
    }
}

#[async_trait]
impl NotificationSender for EmailSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let content = self
            .content_provider
            .get_notification_content(notification)?;
        self.send_inner(&notification.notification_target, content)
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
        let content = self.content_provider.get_grouped_notification_content(
            network_id,
            notification_type_code,
            channel,
            notifications,
        )?;
        self.send_inner(target, content).await
    }
}

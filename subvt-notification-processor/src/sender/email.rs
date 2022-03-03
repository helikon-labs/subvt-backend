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
    pub fn new() -> anyhow::Result<EmailSender> {
        let mailer = Mailer::relay(&CONFIG.notification_sender.email_smtp_server_url)?
            .credentials(Credentials::new(
                CONFIG.notification_sender.email_account.clone(),
                CONFIG.notification_sender.email_password.clone(),
            ))
            // .port(config.notification_sender.email_smtp_server_tls_port)
            .build();
        let content_provider = ContentProvider::new(&CONFIG.notification_sender.template_dir_path)?;
        Ok(EmailSender {
            mailer,
            content_provider,
        })
    }
}

#[async_trait]
impl NotificationSender for EmailSender {
    async fn send(&self, notification: &Notification) -> anyhow::Result<String> {
        let (subject, text_body, html_body) =
            self.content_provider.get_email_content(notification)?;
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

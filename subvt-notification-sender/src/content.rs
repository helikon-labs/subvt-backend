//! Templated notification content provider.

use subvt_types::app::{Block, Notification, NotificationTypeCode, NotificationTypeCode::*};
use tera::{Context, Tera};

use crate::CONFIG;

/// Provider struct. Has separate renderers for separate text notification channels.
/// Expects the `template` folder in this crate to be in the same folder as the executable.
pub struct ContentProvider {
    email_renderer: Tera,
    push_notification_renderer: Tera,
    _sms_renderer: Tera,
    telegram_renderer: Tera,
}

impl ContentProvider {
    pub fn new(template_dir_path: &str) -> anyhow::Result<ContentProvider> {
        Ok(ContentProvider {
            email_renderer: {
                Tera::new(&format!(
                    "{}{}email{}*.txt",
                    template_dir_path,
                    std::path::MAIN_SEPARATOR,
                    std::path::MAIN_SEPARATOR,
                ))?
            },
            push_notification_renderer: {
                Tera::new(&format!(
                    "{}{}push_notification{}*.txt",
                    template_dir_path,
                    std::path::MAIN_SEPARATOR,
                    std::path::MAIN_SEPARATOR,
                ))?
            },
            telegram_renderer: {
                Tera::new(&format!(
                    "{}{}telegram{}*.html",
                    template_dir_path,
                    std::path::MAIN_SEPARATOR,
                    std::path::MAIN_SEPARATOR,
                ))?
            },
            _sms_renderer: {
                Tera::new(&format!(
                    "{}{}sms{}*.txt",
                    template_dir_path,
                    std::path::MAIN_SEPARATOR,
                    std::path::MAIN_SEPARATOR,
                ))?
            },
        })
    }
}

impl ContentProvider {
    pub(crate) fn get_email_content(
        &self,
        notification: &Notification,
    ) -> anyhow::Result<(String, String, String)> {
        let (subject, text_body, html_body) =
            match NotificationTypeCode::from(notification.notification_type_code.as_ref()) {
                ChainValidatorBlockAuthorship => {
                    let mut context = Context::new();
                    context.insert("chain", &CONFIG.substrate.chain);
                    context.insert(
                        "validator_address",
                        &notification.validator_account_id.to_ss58_check(),
                    );
                    context.insert(
                        "validator_display",
                        &if let Some(account) = &notification.get_account()? {
                            account.get_display_or_condensed_address(None)
                        } else {
                            notification.validator_account_id.to_ss58_check()
                        },
                    );
                    let block: Block =
                        serde_json::from_str(notification.data_json.as_ref().unwrap())?;
                    context.insert("block_number", &block.number);
                    let subject = self.email_renderer.render(
                        &format!("{}_subject.txt", notification.notification_type_code),
                        &context,
                    )?;
                    let text_body = self.email_renderer.render(
                        &format!("{}_body_text.txt", notification.notification_type_code),
                        &context,
                    )?;
                    let html_body = self.email_renderer.render(
                        &format!("{}_body_html.txt", notification.notification_type_code),
                        &context,
                    )?;
                    (subject, text_body, html_body)
                }
                _ => todo!(
                    "Email content not yet ready for {}.",
                    notification.notification_type_code
                ),
            };
        Ok((subject, text_body, html_body))
    }

    pub(crate) fn get_push_notification_content(
        &self,
        notification: &Notification,
    ) -> anyhow::Result<String> {
        let message = match NotificationTypeCode::from(notification.notification_type_code.as_ref())
        {
            ChainValidatorBlockAuthorship => {
                let mut context = Context::new();
                context.insert(
                    "validator_display",
                    &if let Some(account) = &notification.get_account()? {
                        account.get_display_or_condensed_address(None)
                    } else {
                        notification.validator_account_id.to_ss58_check()
                    },
                );
                let block: Block = serde_json::from_str(notification.data_json.as_ref().unwrap())?;
                context.insert("block_number", &block.number);
                self.push_notification_renderer.render(
                    &format!("{}_subject.txt", notification.notification_type_code),
                    &context,
                )?
            }
            _ => todo!(
                "Push notification content not yet ready for {}.",
                notification.notification_type_code
            ),
        };
        Ok(message)
    }

    pub(crate) fn get_telegram_content(
        &self,
        notification: &Notification,
    ) -> anyhow::Result<String> {
        let message = match NotificationTypeCode::from(notification.notification_type_code.as_ref())
        {
            ChainValidatorBlockAuthorship => {
                let mut context = Context::new();
                context.insert(
                    "validator_display",
                    &if let Some(account) = &notification.get_account()? {
                        account.get_display_or_condensed_address(None)
                    } else {
                        notification.validator_account_id.to_ss58_check()
                    },
                );
                context.insert("network", &CONFIG.substrate.chain);
                let block: Block = serde_json::from_str(notification.data_json.as_ref().unwrap())?;
                context.insert("block_number", &block.number);
                self.telegram_renderer.render(
                    &format!("{}.html", notification.notification_type_code),
                    &context,
                )?
            }
            _ => todo!(
                "Push notification content not yet ready for {}.",
                notification.notification_type_code
            ),
        };
        Ok(message)
    }
}

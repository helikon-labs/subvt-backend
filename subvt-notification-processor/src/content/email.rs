use crate::{ContentProvider, CONFIG};
use subvt_types::app::{Block, Notification, NotificationTypeCode, NotificationTypeCode::*};
use tera::Context;

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
}

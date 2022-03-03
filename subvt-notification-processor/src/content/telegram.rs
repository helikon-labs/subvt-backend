use crate::{ContentProvider, CONFIG};
use subvt_types::app::{Block, Notification, NotificationTypeCode, NotificationTypeCode::*};
use tera::Context;

impl ContentProvider {
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

use subvt_config::Config;
use subvt_types::app::{Block, Notification, NotificationTypeCode, NotificationTypeCode::*};
use tera::{Context, Tera};

pub struct ContentProvider {
    email_renderer: Tera,
    _push_notification_renderer: Tera,
    _sms_renderer: Tera,
    _instant_message_renderer: Tera,
}

impl ContentProvider {
    pub fn new() -> anyhow::Result<ContentProvider> {
        Ok(ContentProvider {
            email_renderer: { Tera::new("template/email/*.txt")? },
            _instant_message_renderer: { Tera::new("template/instant_message/*.txt")? },
            _push_notification_renderer: { Tera::new("template/push_notification/*.txt")? },
            _sms_renderer: { Tera::new("template/sms/*.txt")? },
        })
    }
}

impl ContentProvider {
    pub(crate) fn get_email_content_for_notification(
        &self,
        config: &Config,
        notification: &Notification,
    ) -> anyhow::Result<(String, String, String)> {
        let (subject, text_body, html_body) =
            match NotificationTypeCode::from(notification.notification_type_code.as_ref()) {
                ChainValidatorOfflineOffence => todo!("Not yet implemented."),
                ChainValidatorNewNomination => todo!("Not yet implemented."),
                ChainValidatorLostNomination => todo!("Not yet implemented."),
                ChainValidatorNominationAmountChange => todo!("Not yet implemented."),
                ChainValidatorChilled => todo!("Not yet implemented."),
                ChainValidatorActive => todo!("Not yet implemented."),
                ChainValidatorActiveNextSession => todo!("Not yet implemented."),
                ChainValidatorInactive => todo!("Not yet implemented."),
                ChainValidatorInactiveNextSession => todo!("Not yet implemented."),
                ChainValidateExtrinsic => todo!("Not yet implemented."),
                ChainValidatorUnclaimedPayout => todo!("Not yet implemented."),
                ChainValidatorBlockAuthorship => {
                    let mut context = Context::new();
                    context.insert("chain", &config.substrate.chain);
                    context.insert(
                        "validator_address",
                        &notification.validator_account_id.to_ss58_check(),
                    );
                    context.insert(
                        "validator_display",
                        &if let Some(account) = &notification.get_account()? {
                            account.to_string()
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
                TelemetryValidatorOffline => todo!("Not yet implemented."),
                TelemetryValidatorBinaryOutOfDate => todo!("Not yet implemented."),
                TelemetryValidatorPeerCountLow => todo!("Not yet implemented."),
                TelemetryValidatorTooManyTxsInQueue => todo!("Not yet implemented."),
                TelemetryValidatorLagging => todo!("Not yet implemented."),
                TelemetryValidatorFinalityLagging => todo!("Not yet implemented."),
                TelemetryValidatorDownloadBwLow => todo!("Not yet implemented."),
                TelemetryValidatorUploadBwLow => todo!("Not yet implemented."),
                OneKVValidatorRankChange => todo!("Not yet implemented."),
                OneKVValidatorValidityChange => todo!("Not yet implemented."),
            };
        Ok((subject, text_body, html_body))
    }
}

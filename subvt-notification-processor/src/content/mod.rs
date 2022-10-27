//! Templated notification content provider.
use crate::content::context::{get_grouped_renderer_context, get_renderer_context};
use crate::CONFIG;
use rustc_hash::FxHashMap as HashMap;
use subvt_types::app::{
    notification::{Notification, NotificationChannel},
    Network,
};
use tera::Tera;

pub(crate) mod context;

#[derive(Debug)]
pub struct NotificationContent {
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}

/// Provider struct. Has separate renderers for separate text notification channels.
/// Expects the `template` folder in this crate to be in the same folder as the executable.
#[derive(Clone)]
pub struct ContentProvider {
    network_map: HashMap<u32, Network>,
    renderer_map: HashMap<NotificationChannel, Tera>,
}

fn get_tera(folder_name: &str) -> anyhow::Result<Tera> {
    Ok(Tera::new(&format!(
        "{}{}{}{}*.*",
        CONFIG.notification_processor.template_dir_path,
        std::path::MAIN_SEPARATOR,
        folder_name,
        std::path::MAIN_SEPARATOR,
    ))?)
}

impl ContentProvider {
    pub fn get_grouped_notification_content(
        &self,
        network_id: u32,
        notification_type_code: &str,
        channel: &NotificationChannel,
        notifications: &[Notification],
    ) -> anyhow::Result<NotificationContent> {
        match self.renderer_map.get(channel) {
            Some(renderer) => {
                let network = self
                    .network_map
                    .get(&network_id)
                    .unwrap_or_else(|| panic!("Cannot find network with id {network_id}."));
                let context =
                    get_grouped_renderer_context(network, notification_type_code, notifications)?;
                let notification_content = NotificationContent {
                    subject: renderer
                        .render(
                            &format!("{notification_type_code}_grouped_subject.txt"),
                            &context,
                        )
                        .ok(),
                    body_text: renderer
                        .render(&format!("{notification_type_code}_grouped.txt"), &context)
                        .ok(),
                    body_html: renderer
                        .render(&format!("{notification_type_code}_grouped.html"), &context)
                        .ok(),
                };
                Ok(notification_content)
            }
            None => panic!("No renderer for notification channel: {channel}"),
        }
    }

    pub fn get_notification_content(
        &self,
        notification: &Notification,
    ) -> anyhow::Result<NotificationContent> {
        match self.renderer_map.get(&notification.notification_channel) {
            Some(renderer) => {
                let network = self
                    .network_map
                    .get(&notification.network_id)
                    .unwrap_or_else(|| {
                        panic!("Cannot find network with id {}.", notification.network_id)
                    });
                let context = get_renderer_context(network, notification)?;
                let notification_content = NotificationContent {
                    subject: renderer
                        .render(
                            &format!("{}_subject.txt", notification.notification_type_code),
                            &context,
                        )
                        .ok(),
                    body_text: renderer
                        .render(
                            &format!("{}.txt", notification.notification_type_code),
                            &context,
                        )
                        .ok(),
                    body_html: renderer
                        .render(
                            &format!("{}.html", notification.notification_type_code),
                            &context,
                        )
                        .ok(),
                };
                Ok(notification_content)
            }
            None => panic!(
                "No renderer for notification channel: {}",
                notification.notification_channel
            ),
        }
    }

    pub fn new(network_map: HashMap<u32, Network>) -> anyhow::Result<ContentProvider> {
        let mut renderer_map = HashMap::default();
        renderer_map.insert(NotificationChannel::APNS, get_tera("push_notification")?);
        renderer_map.insert(NotificationChannel::Email, get_tera("email")?);
        renderer_map.insert(NotificationChannel::FCM, get_tera("push_notification")?);
        renderer_map.insert(NotificationChannel::Telegram, get_tera("telegram")?);
        renderer_map.insert(NotificationChannel::SMS, get_tera("push_notification")?);
        Ok(ContentProvider {
            network_map,
            renderer_map,
        })
    }
}

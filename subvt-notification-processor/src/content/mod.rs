//! Templated notification content provider.

use tera::Tera;

pub(crate) mod email;
pub(crate) mod push_notification;
pub(crate) mod telegram;

/// Provider struct. Has separate renderers for separate text notification channels.
/// Expects the `template` folder in this crate to be in the same folder as the executable.
pub struct ContentProvider {
    email_renderer: Tera,
    push_notification_renderer: Tera,
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
        })
    }
}

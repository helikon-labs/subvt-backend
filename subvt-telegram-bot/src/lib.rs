//! Telegram bot. Former 1KV Telegram Bot migrated to SubVT.

use async_trait::async_trait;
use futures::StreamExt;
use lazy_static::lazy_static;
use log::{error, info};
use subvt_config::Config;
use subvt_service_common::Service;
use telegram_bot::*;
use tera::{Context, Tera};

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct TelegramBot;

#[async_trait(?Send)]
impl Service for TelegramBot {
    async fn run(&'static self) -> anyhow::Result<()> {
        info!("Telegram bot has started.");
        let api = Api::new(&CONFIG.notification_sender.telegram_token);
        let mut stream = api.stream();
        let renderer = Tera::new(&format!(
            "{}{}telegram{}dialog{}*.html",
            CONFIG.notification_sender.template_dir_path,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
        ))?;

        while let Some(update_result) = stream.next().await {
            match update_result {
                Ok(update) => match update.kind {
                    UpdateKind::Message(message) => {
                        println!(
                            "update message received :: chat id :: {}",
                            message.chat.id()
                        );
                        match message.kind {
                            MessageKind::Text { ref data, .. } => {
                                println!("text :: <{}>: {}", &message.from.first_name, data);
                                let content =
                                    renderer.render("introduction.html", &Context::new())?;
                                api.send(message.chat.text(content).parse_mode(ParseMode::Html))
                                    .await?;
                            }
                            MessageKind::GroupChatCreated => {
                                println!("group chat created");
                            }
                            _ => (),
                        }
                    }
                    UpdateKind::ChannelPost(post) => {
                        println!("channel post :: chat id :: {}", post.chat.id);
                    }
                    UpdateKind::Unknown => {
                        println!("unknown update type");
                    }
                    _ => (),
                },
                Err(error) => error!("Error while receiving update: {:?}", error),
            }
        }
        Ok(())
    }
}

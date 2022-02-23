use crate::TelegramBotError;
use frankenstein::{AsyncApi, AsyncTelegramApi, Message, MethodResponse, SendMessageParamsBuilder};
use subvt_config::Config;
use subvt_types::subvt::ValidatorDetails;
use tera::{Context, Tera};

fn get_condensed_address(address: &str) -> String {
    format!("{}...{}", &address[..6], &address[(address.len() - 6)..],)
}

pub enum MessageType {
    Intro,
    BadRequest,
    UnknownCommand(String),
    InvalidAddress(String),
    InvalidAddressTryAgain(String),
    ValidatorNotFound(String),
    ValidatorExistsOnChat(String),
    ValidatorAdded {
        network: String,
        address: String,
        validator_details: Box<ValidatorDetails>,
    },
    AddValidator,
}

impl MessageType {
    pub fn get_content(&self, renderer: &Tera) -> String {
        let mut context = Context::new();
        let template_name = match self {
            Self::Intro => "introduction.html",
            Self::BadRequest => "bad_request.html",
            Self::UnknownCommand(command) => {
                context.insert("command", command);
                "unknown_command.html"
            }
            Self::InvalidAddress(address) => {
                context.insert("address", address);
                "invalid_address.html"
            }
            Self::InvalidAddressTryAgain(address) => {
                context.insert("address", address);
                "invalid_address_try_again.html"
            }
            Self::ValidatorNotFound(address) => {
                context.insert("condensed_address", &get_condensed_address(address));
                "validator_not_found.html"
            }
            Self::ValidatorExistsOnChat(address) => {
                context.insert("condensed_address", &get_condensed_address(address));
                "validator_exists_on_chat.html"
            }
            Self::ValidatorAdded {
                network,
                address,
                validator_details,
            } => {
                context.insert("network", network);
                context.insert("address", address);
                if let Some(display) = validator_details.get_full_display() {
                    context.insert("display", &display);
                } else {
                    context.insert("display", &get_condensed_address(address));
                }
                "validator_added.html"
            }
            Self::AddValidator => "add_validator.html",
        };
        renderer.render(template_name, &context).unwrap()
    }
}

pub struct Messenger {
    api: AsyncApi,
    renderer: Tera,
}

impl Messenger {
    pub fn new(config: &Config, api: AsyncApi) -> anyhow::Result<Messenger> {
        let renderer = Tera::new(&format!(
            "{}{}telegram{}dialog{}*.html",
            config.notification_sender.template_dir_path,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR,
        ))?;
        Ok(Messenger { api, renderer })
    }
}

impl Messenger {
    pub async fn send_message(
        &self,
        chat_id: i64,
        message_type: MessageType,
    ) -> anyhow::Result<MethodResponse<Message>> {
        let params = SendMessageParamsBuilder::default()
            .chat_id(chat_id)
            .text(message_type.get_content(&self.renderer))
            .parse_mode("html")
            .disable_web_page_preview(true)
            .build()
            .unwrap();
        match self.api.send_message(&params).await {
            Ok(response) => Ok(response),
            Err(error) => Err(TelegramBotError::Error(format!("{:?}", error)).into()),
        }
    }
}

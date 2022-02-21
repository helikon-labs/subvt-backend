//! See `./lib.rs` for details.

use lazy_static::lazy_static;
use subvt_service_common::Service;
use subvt_telegram_bot::TelegramBot;

lazy_static! {
    static ref SERVICE: TelegramBot = TelegramBot::default();
}

#[tokio::main]
async fn main() {
    SERVICE.start().await;
}

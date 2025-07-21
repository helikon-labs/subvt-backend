//! Test utilities.

use crate::messenger::MockMessenger;
use crate::{TelegramBot, CONFIG};
use frankenstein::client_reqwest::Bot;
use rand::Rng;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_persistence::redis::Redis;
use subvt_types::crypto::AccountId;

pub mod data;

pub async fn new_test_bot(messenger: MockMessenger) -> anyhow::Result<TelegramBot<MockMessenger>> {
    let app_postgres = PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?;
    let network_postgres =
        PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
    let redis = Redis::new()?;
    let api = Bot::new(&CONFIG.telegram_bot.api_token);
    Ok(TelegramBot {
        app_postgres,
        network_postgres,
        redis,
        api,
        messenger,
    })
}

pub fn get_random_chat_id() -> i64 {
    let mut rng = rand::rng();
    rng.random()
}

pub fn get_random_account_id() -> AccountId {
    let mut rng = rand::rng();
    AccountId::new(rng.random())
}

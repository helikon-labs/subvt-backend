//! Test utilities.
use crate::messenger::MockMessenger;
use crate::{AsyncApi, TelegramBot, CONFIG};
use rand::Rng;
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_persistence::redis::Redis;
use subvt_substrate_client::SubstrateClient;
use subvt_types::crypto::AccountId;

pub mod data;

pub async fn new_test_bot(messenger: MockMessenger) -> anyhow::Result<TelegramBot<MockMessenger>> {
    let app_postgres = PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?;
    let network_postgres =
        PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
    let substrate_client = SubstrateClient::new(&CONFIG).await?;
    let redis = Redis::new()?;
    let api = AsyncApi::new(&CONFIG.telegram_bot.api_token);
    Ok(TelegramBot {
        app_postgres,
        network_postgres,
        redis,
        substrate_client,
        api,
        messenger,
    })
}

pub fn get_random_chat_id() -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn get_random_account_id() -> AccountId {
    let mut rng = rand::thread_rng();
    AccountId::new(rng.gen())
}

use crate::messenger::MockMessenger;
use crate::{AsyncApi, TelegramBot, CONFIG};
use subvt_persistence::postgres::app::PostgreSQLAppStorage;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_persistence::redis::Redis;

mod commands;
mod save_and_restore;
pub mod util;

pub async fn new_test_bot(messenger: MockMessenger) -> anyhow::Result<TelegramBot<MockMessenger>> {
    let app_postgres = PostgreSQLAppStorage::new(&CONFIG, CONFIG.get_app_postgres_url()).await?;
    let network_postgres =
        PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
    let redis = Redis::new()?;
    let api = AsyncApi::new(&CONFIG.telegram_bot.api_token);
    Ok(TelegramBot {
        app_postgres,
        network_postgres,
        redis,
        api,
        messenger,
    })
}

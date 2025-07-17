#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use std::str::FromStr;
use subvt_config::Config;
use subvt_persistence::postgres::network::PostgreSQLNetworkStorage;
use subvt_service_common::Service;
use subvt_types::kline::{BigDecimal, KLine};

mod metrics;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

type KLineRecord = (
    u64,
    String,
    String,
    String,
    String,
    String,
    u64,
    String,
    u32,
    String,
    String,
    String,
);

pub struct KLineUpdater {
    http_client: reqwest::Client,
}

impl Default for KLineUpdater {
    fn default() -> Self {
        let http_client: reqwest::Client = reqwest::Client::builder()
            .gzip(true)
            .brotli(true)
            .timeout(std::time::Duration::from_secs(
                CONFIG.http.request_timeout_seconds,
            ))
            .build()
            .unwrap();
        Self { http_client }
    }
}

#[async_trait(?Send)]
impl Service for KLineUpdater {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (
            CONFIG.metrics.host.as_str(),
            CONFIG.metrics.kline_updater_port,
        )
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        let target_ticker = "USDT";
        let sleep_seconds = CONFIG.kline_updater.sleep_seconds;
        log::info!(
            "KLine updater has started with {} seconds sleep period.",
            sleep_seconds
        );
        let postgres =
            PostgreSQLNetworkStorage::new(&CONFIG, CONFIG.get_network_postgres_url()).await?;
        loop {
            let now: DateTime<Utc> = Utc::now();
            let begin_date = NaiveDate::from_ymd_opt(
                CONFIG.kline_updater.begin_year,
                CONFIG.kline_updater.begin_month,
                CONFIG.kline_updater.begin_day,
            )
            .unwrap();
            let mut day = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
            loop {
                day = day.checked_sub_days(Days::new(1)).unwrap();
                let start_of_day = NaiveDateTime::from(day);
                let year = day.year();
                let month = day.month();
                let month_padded = if month < 10 {
                    format!("0{month}")
                } else {
                    month.to_string()
                };
                let day = day.day();
                let day_padded = if day < 10 {
                    format!("0{day}")
                } else {
                    day.to_string()
                };
                log::info!("Process {day_padded}-{month_padded}-{year}.");
                if start_of_day.date().lt(&begin_date) {
                    log::info!(
                        "{day}.{month}.{year} is before the updater begin date {}.{}.{}.",
                        CONFIG.kline_updater.begin_day,
                        CONFIG.kline_updater.begin_month,
                        CONFIG.kline_updater.begin_year,
                    );
                    break;
                }
                let timestamp = start_of_day.and_utc().timestamp_millis();
                if postgres
                    .kline_exists(timestamp, &CONFIG.substrate.token_ticker, target_ticker)
                    .await?
                {
                    log::info!(
                        "{}-{} k-line record exists for {day}.{month}.{year}.",
                        CONFIG.substrate.token_ticker,
                        target_ticker,
                    );
                    continue;
                }
                let pair = format!("{}{}", CONFIG.substrate.token_ticker, target_ticker);

                // https://api.binance.com/api/v3/klines?symbol=KSMUSDT&interval=1d&limit=1&startTime=1732838400000
                let url = format!("https://api.binance.com/api/v3/klines?symbol={pair}&interval=1d&limit=1&startTime={timestamp}");
                let response = self.http_client.get(&url).send().await?;
                let records: Vec<KLineRecord> = response.json().await?;
                let fields = records.first().unwrap();
                // save record
                let kline = KLine {
                    id: 0,
                    open_time: fields.0,
                    source_ticker: CONFIG.substrate.token_ticker.clone(),
                    target_ticker: target_ticker.to_string(),
                    open: BigDecimal::from_str(&fields.1)?,
                    high: BigDecimal::from_str(&fields.2)?,
                    low: BigDecimal::from_str(&fields.3)?,
                    close: BigDecimal::from_str(&fields.4)?,
                    volume: BigDecimal::from_str(&fields.5)?,
                    close_time: fields.6,
                    quote_volume: BigDecimal::from_str(&fields.7)?,
                    count: fields.8,
                    taker_buy_volume: BigDecimal::from_str(&fields.9)?,
                    taker_buy_quote_volume: BigDecimal::from_str(&fields.10)?,
                };
                postgres.save_kline(&kline).await?;
                log::info!(
                    "Saved {}-{target_ticker} {day_padded}-{month_padded}-{year}.",
                    CONFIG.substrate.token_ticker,
                );
            }
            // publish metrics
            metrics::kline_count().set(postgres.get_kline_count().await? as i64);
            log::info!("K-line updater completed. Will sleep for {sleep_seconds} seconds",);
            tokio::time::sleep(std::time::Duration::from_secs(sleep_seconds)).await;
        }
    }
}

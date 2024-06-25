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

#[derive(Default)]
pub struct KLineUpdater {}

impl KLineUpdater {}

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
                // start from 2 days prior to today
                day = day.checked_sub_days(Days::new(2)).unwrap();
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
                let zip_file_name =
                    format!("{}-1d-{}-{}-{}.zip", pair, year, month_padded, day_padded,);
                let csv_file_name =
                    format!("{}-1d-{}-{}-{}.csv", pair, year, month_padded, day_padded,);
                let zip_file_local_path =
                    format!("{}/{}", CONFIG.kline_updater.tmp_dir_path, zip_file_name,);
                let csv_file_local_path =
                    format!("{}/{}", CONFIG.kline_updater.tmp_dir_path, csv_file_name,);
                if !std::path::Path::new(&zip_file_local_path).is_file() {
                    let url = format!("https://data.binance.vision/data/spot/daily/klines/{pair}/1d/{zip_file_name}");
                    log::info!("Downloading file {url}.");
                    let response = reqwest::get(url).await?;
                    let mut zip_file = std::fs::File::create(&zip_file_local_path)?;
                    let mut content = std::io::Cursor::new(response.bytes().await?);
                    std::io::copy(&mut content, &mut zip_file)?;
                    log::info!("Download complete. Zip file saved at {zip_file_local_path}.");
                } else {
                    log::info!("Zip file exists.");
                }
                // unzip
                if !std::path::Path::new(&csv_file_local_path).is_file() {
                    let target_dir = std::path::PathBuf::from(&CONFIG.kline_updater.tmp_dir_path);
                    let zip_source = std::fs::read(&zip_file_local_path)?;
                    zip_extract::extract(std::io::Cursor::new(zip_source), &target_dir, true)?;
                    log::info!("Zip file extracted.");
                } else {
                    log::info!("CSV file exists.");
                }
                // read file
                let csv_content = std::fs::read_to_string(&csv_file_local_path)?;
                let fields: Vec<&str> = csv_content.split(",").collect();
                // save record
                let kline = KLine {
                    id: 0,
                    open_time: fields[0].parse()?,
                    source_ticker: CONFIG.substrate.token_ticker.clone(),
                    target_ticker: target_ticker.to_string(),
                    open: BigDecimal::from_str(fields[1])?,
                    high: BigDecimal::from_str(fields[2])?,
                    low: BigDecimal::from_str(fields[3])?,
                    close: BigDecimal::from_str(fields[4])?,
                    volume: BigDecimal::from_str(fields[5])?,
                    close_time: fields[6].parse()?,
                    quote_volume: BigDecimal::from_str(fields[7])?,
                    count: fields[8].parse()?,
                    taker_buy_volume: BigDecimal::from_str(fields[9])?,
                    taker_buy_quote_volume: BigDecimal::from_str(fields[10])?,
                };
                postgres.save_kline(&kline).await?;
                log::info!(
                    "Saved {}-{target_ticker} {day_padded}-{month_padded}-{year}.",
                    CONFIG.substrate.token_ticker,
                );
                // delete temp files
                let _ = std::fs::remove_file(&zip_file_local_path);
                let _ = std::fs::remove_file(&csv_file_local_path);
            }
            // publish metrics
            metrics::kline_count().set(postgres.get_kline_count().await? as i64);
            // publish total count
            log::info!(
                "K-line updater completed. Will sleep for {} seconds",
                sleep_seconds
            );
            tokio::time::sleep(std::time::Duration::from_secs(sleep_seconds)).await;
        }
    }
}

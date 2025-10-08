//! SubVT runtime configuration.

use serde::Deserialize;
use std::fmt;

const DEFAULT_CONFIG_DIR: &str = "./config";
const DEV_CONFIG_DIR: &str = "../_config";
const DEFAULT_NETWORK: &str = "kusama";

/// Runtime environment.
#[derive(Clone, Debug, Deserialize)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "Development"),
            Environment::Test => write!(f, "Test"),
            Environment::Production => write!(f, "Production"),
        }
    }
}

impl From<&str> for Environment {
    fn from(env: &str) -> Self {
        match env.to_lowercase().as_str() {
            "testing" | "test" => Environment::Test,
            "production" | "prod" => Environment::Production,
            "development" | "dev" => Environment::Development,
            _ => panic!("Unknown environment: {env}"),
        }
    }
}

/// Common configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct CommonConfig {
    /// Wait this many seconds before retrying to recover from a fatal error condition.
    pub recovery_retry_seconds: u64,
}

/// Substrate configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct SubstrateConfig {
    /// Name of the chain (`kusama`, `polkadot`, `darwinia`, etc.).
    pub chain: String,
    /// Display name of the chain (`Kusama`, `Polkadot`, `Darwinia`, etc.).
    pub chain_display: String,
    /// Hash of the genesis block of the chain.
    pub chain_genesis_hash: String,
    /// Number of epochs per era on the chain.
    pub epochs_per_era: u16,
    /// Node WebSocket RPC URL (e.g. `wss://kusama-rpc.polkadot.io` for Kusama).
    pub rpc_url: String,
    /// Identity chain RPC URL.
    pub asset_hub_rpc_url: String,
    /// Identity chain RPC URL.
    pub people_rpc_url: String,
    /// RPC connection timeout in seconds.
    pub connection_timeout_seconds: u64,
    /// RPC request timeout in seconds.
    pub request_timeout_seconds: u64,
    /// Substrate network id for internal use.
    pub network_id: u32,
    /// Ticker for the network utility token (KSM, DOT, etc.).
    pub token_ticker: String,
    /// Number of decimals for the network utility token.
    pub token_decimals: usize,
    /// Number of decimal points in the formatted amount (e.g. 4 in 14.7983 KSM).
    pub token_format_decimal_points: usize,
}

/// Log configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct LogConfig {
    /// Log level for SubVT modules.
    pub subvt_level: String,
    /// Log level for all other modules.
    pub other_level: String,
}

/// RPC server configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct RPCConfig {
    /// Host IP address.
    pub host: String,
    /// Network status WS RPC server TCP port.
    pub network_status_port: String,
    /// Active validator list WS RPC server TCP port.
    pub active_validator_list_port: u16,
    /// Inactive validator list WS RPC server TCP port.
    pub inactive_validator_list_port: u16,
    /// Validator details WS RPC server TCP port.
    pub validator_details_port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HTTPConfig {
    pub request_timeout_seconds: u64,
    pub service_host: String,
    /// Report REST service TCP port.
    pub report_service_port: u16,
    /// Application REST service TCP port.
    pub app_service_port: u16,
}

/// Redis configuration. Redis is utilized as in-memory buffer storage for real-time
/// validator list and network status data.
#[derive(Clone, Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub kusama_url: String,
    pub polkadot_url: String,
    pub westend_url: String,
}

/// PostgreSQL configuration. PostgreSQL is used for historical indexed blockchain data storage.
#[derive(Clone, Debug, Deserialize)]
pub struct PostgreSQLConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: String,
    pub pool_max_connections: u32,
    pub connection_timeout_seconds: u64,
}

/// SubVT block processor configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct BlockProcessorConfig {
    /// Indexing starts at this block, indexes all blocks up to
    /// current blocks, then continues with every new block.
    pub relay_start_block_number: u64,
    pub asset_hub_start_block_number: u64,
}

/// Validator list updater configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct ValidatorListUpdaterConfig {
    /// Keep this many records in Redis for the past validator list updates.
    pub history_record_depth: u64,
    pub db_fetch_batch_size: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DNConfig {
    pub candidate_history_record_count: u64,
    pub data_endpoint: String,
    pub refresh_seconds: u64,
}

/// Report service configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct ReportConfig {
    pub max_era_index_range: u32,
    pub max_session_index_range: u32,
}

/// Telemetry processor configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct TelemetryConfig {
    pub websocket_url: String,
}

/// Notification generator configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct NotificationGeneratorConfig {
    pub unclaimed_payout_check_delay_hours: u32,
}

/// Notification sender configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct NotificationProcessorConfig {
    pub sleep_millis: u64,
    pub email_from: String,
    pub email_reply_to: String,
    pub email_account: String,
    pub email_password: String,
    pub email_smtp_server_url: String,
    pub email_smtp_server_tls_port: u16,
    // Apple Push Notification Service
    pub apns_key_path: String,
    pub apns_key_id: String,
    pub apns_team_id: String,
    pub apns_topic: String,
    pub apns_is_production: bool,
    // Firebase Cloud Messaging
    pub fcm_api_key: String,
    // Kusama Telegram bot API token
    pub kusama_telegram_api_token: String,
    // Kusama Telegram bot API token
    pub polkadot_telegram_api_token: String,
    // where the template files reside
    pub template_dir_path: String,
}

/// Telegram bot config.
#[derive(Clone, Debug, Deserialize)]
pub struct TelegramBotConfig {
    pub api_token: String,
    admin_chat_ids: String,
    pub max_validators_per_chat: u16,
    pub username: String,
}

impl TelegramBotConfig {
    pub fn get_admin_chat_ids(&self) -> Vec<i64> {
        self.admin_chat_ids
            .replace(' ', "")
            .split(',')
            .map(|item| item.parse::<i64>().unwrap())
            .collect()
    }
}

/// Prometheus metrics config.
#[derive(Clone, Debug, Deserialize)]
pub struct MetricsConfig {
    pub host: String,
    pub block_processor_port: u16,
    pub validator_list_updater_port: u16,
    pub validator_details_server_port: u16,
    pub active_validator_list_server_port: u16,
    pub inactive_validator_list_server_port: u16,
    pub network_status_updater_port: u16,
    pub network_status_server_port: u16,
    pub onekv_updater_port: u16,
    pub telemetry_processor_port: u16,
    pub report_service_port: u16,
    pub notification_generator_port: u16,
    pub notification_processor_port: u16,
    pub telegram_bot_port: u16,
    pub app_service_port: u16,
    pub referendum_updater_port: u16,
    pub kline_updater_port: u16,
    pub session_validator_performance_updater_port: u16,
}

/// Plotter config.
#[derive(Clone, Debug, Deserialize)]
pub struct PlotterConfig {
    pub tmp_dir_path: String,
    pub font_dir_path: String,
    pub font_sans_serif_family: String,
}

/// Sub.id NFTs API configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct SubIDConfig {
    pub api_url: String,
    pub nfts_path: String,
}

/// App service configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct AppServiceConfig {
    pub user_registration_per_ip_limit_time_window_mins: u16,
    pub user_registration_per_ip_limit: u16,
}

/// Referendum updater configuration - fetches data from Polkassembly.
#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumUpdaterConfig {
    pub refresh_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct KLineUpdaterConfig {
    pub sleep_seconds: u64,
    pub begin_year: i32,
    pub begin_month: u32,
    pub begin_day: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SessionValidatorPerformanceUpdaterConfig {
    pub start_session_index: u64,
    pub sleep_seconds: u64,
}

/// Whole configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub block_processor: BlockProcessorConfig,
    pub validator_list_updater: ValidatorListUpdaterConfig,
    pub env: Environment,
    pub common: CommonConfig,
    pub http: HTTPConfig,
    pub log: LogConfig,
    pub dn: DNConfig,
    pub app_postgres: PostgreSQLConfig,
    pub network_postgres: PostgreSQLConfig,
    pub redis: RedisConfig,
    pub rpc: RPCConfig,
    pub substrate: SubstrateConfig,
    pub report: ReportConfig,
    pub telemetry: TelemetryConfig,
    pub notification_generator: NotificationGeneratorConfig,
    pub notification_processor: NotificationProcessorConfig,
    pub telegram_bot: TelegramBotConfig,
    pub metrics: MetricsConfig,
    pub plotter: PlotterConfig,
    pub sub_id: SubIDConfig,
    pub app_service: AppServiceConfig,
    pub referendum_updater: ReferendumUpdaterConfig,
    pub kline_updater: KLineUpdaterConfig,
    pub session_validator_performance_updater: SessionValidatorPerformanceUpdaterConfig,
}

impl Config {
    pub fn test() -> Result<Self, config::ConfigError> {
        let env = Environment::Test;
        let config = config::Config::builder()
            .set_default("env", env.to_string())?
            .add_source(config::File::with_name(&format!("{DEV_CONFIG_DIR}/base")))
            .add_source(config::File::with_name(&format!(
                "{DEV_CONFIG_DIR}/network/{DEFAULT_NETWORK}"
            )))
            .add_source(config::File::with_name(&format!(
                "{DEV_CONFIG_DIR}/env/{}",
                env.to_string().to_lowercase()
            )))
            .add_source(config::Environment::with_prefix("subvt").separator("__"))
            .build()?;
        config.try_deserialize()
    }

    fn new() -> Result<Self, config::ConfigError> {
        let env = Environment::from(
            std::env::var("SUBVT_ENV")
                .unwrap_or_else(|_| "Production".into())
                .as_str(),
        );
        let network = std::env::var("SUBVT_NETWORK").unwrap_or_else(|_| DEFAULT_NETWORK.into());
        if cfg!(debug_assertions) {
            let config_dir =
                std::env::var("SUBVT_CONFIG_DIR").unwrap_or_else(|_| DEV_CONFIG_DIR.into());
            let config = config::Config::builder()
                .set_default("env", env.to_string())?
                .add_source(config::File::with_name(&format!("{config_dir}/base")))
                .add_source(config::File::with_name(&format!(
                    "{config_dir}/network/{network}",
                )))
                .add_source(config::File::with_name(&format!(
                    "{}/env/{}",
                    config_dir,
                    env.to_string().to_lowercase()
                )))
                .add_source(config::Environment::with_prefix("subvt").separator("__"))
                .build()?;
            config.try_deserialize()
        } else {
            let config_dir =
                std::env::var("SUBVT_CONFIG_DIR").unwrap_or_else(|_| DEFAULT_CONFIG_DIR.into());
            let config = config::Config::builder()
                .set_default("env", env.to_string())?
                .add_source(config::File::with_name(&format!("{config_dir}/base")))
                .add_source(config::File::with_name(&format!(
                    "{config_dir}/network/{network}",
                )))
                .add_source(config::File::with_name(&format!(
                    "{}/env/{}",
                    config_dir,
                    env.to_string().to_lowercase()
                )))
                .add_source(config::Environment::with_prefix("subvt").separator("__"))
                .build()?;
            config.try_deserialize()
        }
    }

    pub fn get_app_postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode=disable",
            self.app_postgres.username,
            self.app_postgres.password,
            self.app_postgres.host,
            self.app_postgres.port,
            self.app_postgres.database_name,
        )
    }

    pub fn get_network_postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode=disable",
            self.network_postgres.username,
            self.network_postgres.password,
            self.network_postgres.host,
            self.network_postgres.port,
            self.network_postgres.database_name,
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().expect("Config can't be loaded.")
    }
}

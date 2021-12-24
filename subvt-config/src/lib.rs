//! SubVT runtime configuration.

use serde::Deserialize;
use std::fmt;

/// Default development configuration file relative path for other SubVT crates/modules.
const DEV_CONFIG_FILE_PATH: &str = "../subvt-config/config/Default.toml";
/// Development configuration folder relative path for other SubVT crates/modules.
const DEV_CONFIG_FILE_PREFIX: &str = "../subvt-config/config/";
/// Production default configuration file should reside in the folder `config` in the same
/// folder as the final executable.
const CONFIG_FILE_PATH: &str = "./config/Default.toml";
/// Production configuration folder should reside in the folder `config` in the same
/// folder as the final executable.
const CONFIG_FILE_PREFIX: &str = "./config/";

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
            _ => panic!("Unknown environment: {}", env),
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
    /// Hash of the genesis block of the chain.
    pub chain_genesis_hash: String,
    /// Node WebSocket RPC URL (e.g. `wss://kusama-rpc.polkadot.io` for Kusama).
    pub rpc_url: String,
    /// RPC connection timeout in seconds.
    pub connection_timeout_seconds: u64,
    /// RPC request timeout in seconds.
    pub request_timeout_seconds: u64,
    /// Substrate network id for internal use.
    pub network_id: u32,
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
    /// TCP port for the live network status WS RPC server.
    pub live_network_status_port: String,
    /// TCP port for the active validator list WS RPC server.
    pub active_validator_list_port: u16,
    /// TCP port for the inactive validator list WS RPC server.
    pub inactive_validator_list_port: u16,
    /// TCP port for the validator details WS RPC server.
    pub validator_details_port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HTTPConfig {
    pub host: String,
    pub report_service_port: u16,
    pub app_service_port: u16,
}

/// Redis configuration. Redis is utilized as in-memory
/// buffer storage for real-time data.
#[derive(Clone, Debug, Deserialize)]
pub struct RedisConfig {
    /// Redis URL.
    pub url: String,
}

/// PostgreSQL configuration. PostgreSQL is used for historical
/// indexed blockchain data storage.
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
    pub start_block_number: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OneKVConfig {
    pub candidate_list_endpoint: String,
    pub candidate_details_endpoint: String,
    pub refresh_seconds: u64,
    pub request_timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReportConfig {
    pub max_era_index_range: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelemetryConfig {
    pub websocket_url: String,
}

/// Whole configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub block_processor: BlockProcessorConfig,
    pub env: Environment,
    pub common: CommonConfig,
    pub http: HTTPConfig,
    pub log: LogConfig,
    pub onekv: OneKVConfig,
    pub app_postgres: PostgreSQLConfig,
    pub network_postgres: PostgreSQLConfig,
    pub redis: RedisConfig,
    pub rpc: RPCConfig,
    pub substrate: SubstrateConfig,
    pub report: ReportConfig,
    pub telemetry: TelemetryConfig,
}

impl Config {
    pub fn test() -> Result<Self, config::ConfigError> {
        let env = Environment::Test;
        let mut c = config::Config::new();
        c.set("env", env.to_string())?;
        c.merge(config::File::with_name(DEV_CONFIG_FILE_PATH))?;
        c.merge(config::File::with_name(&format!(
            "{}{}",
            DEV_CONFIG_FILE_PREFIX, env
        )))?;
        // this makes it so SUBVT_REDIS__URL overrides redis.url
        c.merge(config::Environment::with_prefix("subvt").separator("__"))?;
        c.try_into()
    }

    fn new() -> Result<Self, config::ConfigError> {
        let env = Environment::from(
            std::env::var("SUBVT_ENV")
                .unwrap_or_else(|_| "Production".into())
                .as_str(),
        );
        let mut c = config::Config::new();
        c.set("env", env.to_string())?;
        if cfg!(debug_assertions) {
            c.merge(config::File::with_name(DEV_CONFIG_FILE_PATH))?;
            c.merge(config::File::with_name(&format!(
                "{}{}",
                DEV_CONFIG_FILE_PREFIX, env
            )))?;
        } else {
            c.merge(config::File::with_name(CONFIG_FILE_PATH))?;
            c.merge(config::File::with_name(&format!(
                "{}{}",
                CONFIG_FILE_PREFIX, env
            )))?;
        }
        // this makes it so SUBVT_REDIS__URL overrides redis.url
        c.merge(config::Environment::with_prefix("subvt").separator("__"))?;
        c.try_into()
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

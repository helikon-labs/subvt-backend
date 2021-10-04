//! Logging configuration and initializer.

use env_logger::{Builder, Env, Target, WriteStyle};
use log::LevelFilter;
use std::str::FromStr;

/// Initializes the logging facade using the application configuration reference.
pub fn init(config: &subvt_config::Config) {
    let other_modules_log_level = LevelFilter::from_str(
        config.log.other_level.as_str()
    ).expect("Cannot read log level configuration for outside modules.");
    let log_level = LevelFilter::from_str(
        config.log.subvt_level.as_str()
    ).expect("Cannot read log level configuration for SubVT modules.");
    let mut builder = Builder::from_env(Env::default());
    builder.target(Target::Stdout);
    builder.filter(None, other_modules_log_level);
    builder.filter(Some("subvt_block_indexer"), log_level);
    builder.filter(Some("subvt_inactive_validator_list_server"), log_level);
    builder.filter(Some("subvt_inactive_validators_updater"), log_level);
    builder.filter(Some("subvt_live_network_status_server"), log_level);
    builder.filter(Some("subvt_live_network_status_updater"), log_level);
    builder.filter(Some("subvt_substrate_client"), log_level);
    builder.filter(Some("subvt_types"), log_level);
    builder.write_style(WriteStyle::Always);
    builder.init();
}
#![warn(clippy::disallowed_types)]
use lazy_static::lazy_static;
use subvt_config::Config;

pub mod polkassembly;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

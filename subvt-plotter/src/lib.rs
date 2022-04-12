use lazy_static::lazy_static;
use subvt_config::Config;

mod plotlib;
pub mod rewards;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum PlotterError {
    #[error("Provided data set is empty.")]
    EmptyData,
}

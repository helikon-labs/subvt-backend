use once_cell::sync::Lazy;
use subvt_metrics::registry::IntGauge;

const METRIC_PREFIX: &str = "subvt_referendum_updater";

pub fn last_run_timestamp_ms() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "last_run_timestamp_ms",
            "Timestamp (ms) for the last run",
        )
        .unwrap()
    });
    METER.clone()
}

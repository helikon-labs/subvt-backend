use once_cell::sync::Lazy;
use subvt_metrics::registry::IntGauge;

const METRIC_PREFIX: &str = "subvt_kline_updater";

pub fn kline_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "kline_count",
            "Total number of k-line records",
        )
        .unwrap()
    });
    METER.clone()
}

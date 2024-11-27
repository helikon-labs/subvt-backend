use once_cell::sync::Lazy;
use subvt_metrics::registry::IntGauge;

const _METRIC_PREFIX: &str = "subvt_validator_score_updater";

pub fn _target_finalized_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        subvt_metrics::registry::register_int_gauge(
            _METRIC_PREFIX,
            "target_finalized_block_number",
            "Number of the target finalized block on the node",
        )
        .unwrap()
    });
    METER.clone()
}

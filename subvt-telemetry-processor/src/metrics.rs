use once_cell::sync::OnceCell;
use subvt_metrics::registry::IntGauge;

const METRIC_PREFIX: &str = "subvt_telemetry_processor";

static BEST_BLOCK_NUMBER: OnceCell<IntGauge> = OnceCell::new();
static FINALIZED_BLOCK_NUMBER: OnceCell<IntGauge> = OnceCell::new();
static NODE_COUNT: OnceCell<IntGauge> = OnceCell::new();

pub(crate) fn init() {
    if BEST_BLOCK_NUMBER.get().is_none() {
        let _ = BEST_BLOCK_NUMBER.set(
            subvt_metrics::registry::register_int_gauge(
                METRIC_PREFIX,
                "best_block_number",
                "Number of the network's best block",
            )
            .unwrap(),
        );
    }
    if FINALIZED_BLOCK_NUMBER.get().is_none() {
        let _ = FINALIZED_BLOCK_NUMBER.set(
            subvt_metrics::registry::register_int_gauge(
                METRIC_PREFIX,
                "finalized_block_number",
                "Number of the network's finalized block",
            )
            .unwrap(),
        );
    }
    if NODE_COUNT.get().is_none() {
        let _ = NODE_COUNT.set(
            subvt_metrics::registry::register_int_gauge(
                METRIC_PREFIX,
                "node_count",
                "Number of nodes connected to this Telemetry instance",
            )
            .unwrap(),
        );
    }
}

pub fn best_block_number() -> IntGauge {
    BEST_BLOCK_NUMBER.get().unwrap().clone()
}

pub fn finalized_block_number() -> IntGauge {
    FINALIZED_BLOCK_NUMBER.get().unwrap().clone()
}

pub fn node_count() -> IntGauge {
    NODE_COUNT.get().unwrap().clone()
}

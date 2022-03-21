use once_cell::sync::OnceCell;
use subvt_metrics::registry::IntGauge;

static TARGET_FINALIZED_BLOCK_NUMBER: OnceCell<IntGauge> = OnceCell::new();
static PROCESSED_FINALIZED_BLOCK_NUMBER: OnceCell<IntGauge> = OnceCell::new();
static SUBSCRIPTION_COUNT: OnceCell<IntGauge> = OnceCell::new();

pub(crate) fn init(prefix: &str) {
    if TARGET_FINALIZED_BLOCK_NUMBER.get().is_none() {
        let _ = TARGET_FINALIZED_BLOCK_NUMBER.set(
            subvt_metrics::registry::register_int_gauge(
                prefix,
                "target_finalized_block_number",
                "Number of the target finalized block on the node",
            )
            .unwrap(),
        );
    }
    if PROCESSED_FINALIZED_BLOCK_NUMBER.get().is_none() {
        let _ = PROCESSED_FINALIZED_BLOCK_NUMBER.set(
            subvt_metrics::registry::register_int_gauge(
                prefix,
                "processed_finalized_block_number",
                "Number of the target finalized block on the node",
            )
            .unwrap(),
        );
    }
    if SUBSCRIPTION_COUNT.get().is_none() {
        let _ = SUBSCRIPTION_COUNT.set(
            subvt_metrics::registry::register_int_gauge(
                prefix,
                "subscription_count",
                "Number of the target finalized block on the node",
            )
            .unwrap(),
        );
    }
}

pub fn target_finalized_block_number() -> IntGauge {
    TARGET_FINALIZED_BLOCK_NUMBER.get().unwrap().clone()
}

pub fn processed_finalized_block_number() -> IntGauge {
    PROCESSED_FINALIZED_BLOCK_NUMBER.get().unwrap().clone()
}

pub fn subscription_count() -> IntGauge {
    SUBSCRIPTION_COUNT.get().unwrap().clone()
}

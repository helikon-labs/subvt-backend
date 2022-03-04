use once_cell::sync::Lazy;
use prometheus::opts;
pub use prometheus::{
    core::Collector, proto, Counter, CounterVec, Error, Gauge, GaugeVec, Histogram, HistogramOpts,
    HistogramTimer, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Registry,
};
use std::sync::{Arc, RwLock};

static DEFAULT_REGISTRY: Lazy<Arc<RwLock<Registry>>> =
    Lazy::new(|| Arc::new(RwLock::new(Registry::default())));

pub(crate) fn get_default_registry() -> Registry {
    DEFAULT_REGISTRY.read().unwrap().clone()
}

fn register<C: Collector + 'static>(c: C) -> prometheus::Result<()> {
    get_default_registry().register(Box::new(c))
}

pub fn register_gauge(name: &str, help: &str) -> prometheus::Result<Gauge> {
    let gauge = prometheus::Gauge::new(name, help)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_gauge_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<GaugeVec> {
    let gauge = prometheus::GaugeVec::new(opts!(name, help), label_names)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_int_gauge_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<IntGaugeVec> {
    let gauge = prometheus::IntGaugeVec::new(opts!(name, help), label_names)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_int_counter(name: &str, help: &str) -> prometheus::Result<IntCounter> {
    let gauge = prometheus::IntCounter::new(name, help)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_int_counter_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<IntCounterVec> {
    let gauge = prometheus::IntCounterVec::new(opts!(name, help), label_names)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_int_gauge(name: &str, help: &str) -> prometheus::Result<IntGauge> {
    let gauge = prometheus::IntGauge::new(name, help)?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_histogram(name: &str, help: &str) -> prometheus::Result<Histogram> {
    let gauge = prometheus::Histogram::with_opts(HistogramOpts::new(name, help))?;
    register(gauge.clone())?;
    Ok(gauge)
}

pub fn register_histogram_vec(
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<HistogramVec> {
    let gauge = prometheus::HistogramVec::new(HistogramOpts::new(name, help), label_names)?;
    register(gauge.clone())?;
    Ok(gauge)
}

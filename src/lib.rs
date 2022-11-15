pub mod auth;
pub mod collector;
pub mod web;

use tracing::Level;

pub use self::web::Web;

pub fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));

    log::info!("initialized logging with max level")
}

pub fn init_tracing(max_level: Level) {
    let subscriber = tracing_subscriber::fmt().with_max_level(max_level).finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set global default tracing subscriber");

    log::info!("initialized tracing with max level `{max_level}`");
}

pub fn init_metrics() {
    self::web::METRICS_HANDLE.get_or_init(|| {
        metrics_exporter_prometheus::PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install metrics recorder and exporter")
    });
}

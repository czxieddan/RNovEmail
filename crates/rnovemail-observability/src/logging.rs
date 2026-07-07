use rnovemail_config::{LogFormat, ObservabilityConfig};
use tracing_subscriber::{EnvFilter, fmt};

pub fn init_logging(config: &ObservabilityConfig) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    match config.log_format {
        LogFormat::Compact => init_compact(filter),
        LogFormat::Json => init_json(filter),
    }
}

fn init_compact(filter: EnvFilter) {
    let _ = fmt().with_env_filter(filter).try_init();
}

fn init_json(filter: EnvFilter) {
    let _ = fmt().json().with_env_filter(filter).try_init();
}

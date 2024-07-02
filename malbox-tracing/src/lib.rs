use std::collections::HashMap;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

fn parse_log_filters(log_filters: &str) -> HashMap<String, tracing::Level> {
    let mut log_levels = HashMap::new();

    for filter in log_filters.split(',') {
        let parts: Vec<&str> = filter.split('=').collect();
        if parts.len() == 2 {
            let module = parts[0].trim().to_string();
            let level_str = parts[1].trim().to_lowercase();
            let level = match level_str.as_str() {
                "trace" => tracing::Level::TRACE,
                "debug" => tracing::Level::DEBUG,
                "info" => tracing::Level::INFO,
                "warn" => tracing::Level::WARN,
                "error" => tracing::Level::ERROR,
                _ => continue,
            };
            log_levels.insert(module, level);
        }
    }

    log_levels
}

pub fn init_tracing(log_filter: &str) {
    let log_levels = parse_log_filters(log_filter);

    let subscriber = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::Targets::new().with_targets(
                log_levels
                    .into_iter()
                    .map(|(module, level)| (module, LevelFilter::from_level(level))),
            ),
        );

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}

use anyhow::Context;
use repositories::tasks_repository::TaskEntity;
use scheduler::{TaskScheduler, TaskWorker};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Semaphore};
use tracing::{level_filters::LevelFilter, subscriber};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use malbox_shared::config::load_config;
mod http;
mod repositories;
mod scheduler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await;

    init_tracing(&config.debug.rust_log);

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.postgres.database_url)
        .await
        .unwrap();

    sqlx::migrate!().run(&db).await.unwrap();

    let db_clone = db.clone();

    let (tx, rx) = mpsc::channel::<TaskEntity>(8);

    let scheduler = TaskScheduler::new(tx.clone(), db_clone);
    tokio::spawn(async move {
        scheduler.scheduler().await;
    });

    let worker = TaskWorker::new(rx, 1);
    tokio::spawn(async move {
        worker.worker().await;
    });

    http::serve(config.clone(), db).await?;

    Ok(())
}

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

fn init_tracing(log_filter: &str) {
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

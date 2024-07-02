use malbox_config::malbox::Postgres;
pub use sqlx::error::DatabaseError;
use sqlx::postgres::PgPoolOptions;
pub use sqlx::Error;
pub use sqlx::PgPool;

pub mod repositories;

pub async fn init_database(config: &Postgres) -> sqlx::Pool<sqlx::Postgres> {
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .unwrap();

    sqlx::migrate!().run(&db).await.unwrap();

    db
}

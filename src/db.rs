use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::env;
use std::str::FromStr;

pub type DbPool = SqlitePool;

pub async fn init_db_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:tracking.db".to_string());
    let opts = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
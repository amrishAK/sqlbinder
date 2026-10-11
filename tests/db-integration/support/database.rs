#![allow(dead_code)]

use sqlbinder::context_container::app_settings::AppSettings;
use sqlbinder::db::{QueryExecutor, create_query_executor, get_connection_pool};

pub const SQLITE_SEED: &str = include_str!("../../test-utils/sqlite/seed.sql");
pub const POSTGRES_SEED: &str = include_str!("../../test-utils/postgres/seed.sql");

fn database_settings(config: &str) -> sqlbinder::context_container::DatabaseSettings {
    let raw = toml::from_str(config).expect("integration settings should be valid TOML");
    AppSettings::from_toml_value(raw)
        .expect("integration settings should be valid")
        .database
}

/// Each in-memory pool is limited to one connection so the database stays alive
/// for the whole executor lifetime and is isolated per test.
pub fn sqlite_executor() -> QueryExecutor {
    let settings = database_settings(include_str!("../settings/sqlite.toml"));
    let pool = get_connection_pool(&settings).expect("SQLite pool should be created");

    create_query_executor(&pool)
}

pub async fn seeded_sqlite_executor() -> QueryExecutor {
    let executor = sqlite_executor();

    executor
        .execute_write(SQLITE_SEED)
        .await
        .expect("SQLite seed should be applied");

    executor
}

pub fn postgres_executor() -> QueryExecutor {
    let settings = database_settings(include_str!("../settings/postgres.toml"));
    let pool = get_connection_pool(&settings).expect("PostgreSQL pool should be created");

    create_query_executor(&pool)
}

/// The PostgreSQL seed is idempotent, so re-applying it makes the suite usable
/// even when the compose seed service has not run.
pub async fn seeded_postgres_executor() -> QueryExecutor {
    let executor = postgres_executor();

    executor
        .execute_write(POSTGRES_SEED)
        .await
        .expect("PostgreSQL seed should be applied");

    executor
}

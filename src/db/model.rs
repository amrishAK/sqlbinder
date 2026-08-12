use super::pg::model::PostgresPool;
use super::sqlite::model::SqlitePool;

pub enum DbConnectionPool {
    Postgres(PostgresPool),
    Sqlite(SqlitePool),
}


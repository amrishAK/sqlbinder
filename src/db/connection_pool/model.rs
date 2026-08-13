use super::super::pg::model::PostgresPool;
use super::super::sqlite::model::SqlitePool;

#[derive(Debug, Clone)]
pub enum DbConnectionPool {
    Postgres(PostgresPool),
    Sqlite(SqlitePool),
}
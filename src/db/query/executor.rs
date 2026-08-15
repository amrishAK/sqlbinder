use super::error::DbQueryError;
use super::model::{BackendQueryExecutor, QueryExecutor, QueryRows};
use crate::db::DbConnectionPool;
use crate::db::pg::query::PostgresQueryExecutor;
use crate::db::sqlite::query::SqliteQueryExecutor;
use std::sync::Arc;

impl QueryExecutor {
    pub fn new(pool: DbConnectionPool) -> Self {
        let backend_executor = match pool {
            DbConnectionPool::Postgres(pool) => Arc::new(PostgresQueryExecutor::new(pool))
                as Arc<dyn BackendQueryExecutor>,
            DbConnectionPool::Sqlite(pool) => Arc::new(SqliteQueryExecutor::new(pool))
                as Arc<dyn BackendQueryExecutor>,
        };

        Self { backend_executor }
    }

    pub async fn execute_write(&self, sql: &str) -> Result<u64, DbQueryError> {
        self.backend_executor.execute_backend_write(sql).await
    }

    pub async fn execute_read(&self, sql: &str) -> Result<QueryRows, DbQueryError> {
        self.backend_executor.execute_backend_read(sql).await
    }
}


#[cfg(test)]
mod tests {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    use super::*;
    use crate::db::DbValue;

    fn in_memory_executor() -> QueryExecutor {
        let pool = crate::db::SqlitePool::new(
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect_lazy_with(SqliteConnectOptions::new().in_memory(true)),
        );

        QueryExecutor::new(DbConnectionPool::Sqlite(pool))
    }

    #[tokio::test]
    async fn execute_upsert_returns_affected_rows_for_insert_success() {
        let executor = in_memory_executor();

        executor
            .execute_write("CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .expect("expected create table query to succeed");

        let result = executor
            .execute_write("INSERT INTO users(name) VALUES('alice')")
            .await
            .expect("expected insert query to succeed");

        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn execute_read_returns_rows_for_select_success() {
        let executor = in_memory_executor();

        executor
            .execute_write("CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .expect("expected create table query to succeed");
        executor
            .execute_write("INSERT INTO users(name) VALUES('alice')")
            .await
            .expect("expected insert query to succeed");

        let rows = executor
            .execute_read("SELECT id, name FROM users")
            .await
            .expect("expected select query to succeed");

        assert_eq!(rows.columns, vec!["id".to_owned(), "name".to_owned()]);
        assert_eq!(rows.rows.len(), 1);
        assert_eq!(
            rows.rows[0],
            vec![DbValue::Int64(1), DbValue::String("alice".to_owned())]
        );
    }
}
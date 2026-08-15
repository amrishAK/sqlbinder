use crate::db::DbConnectionPool;
use super::model::QueryExecutor;

pub fn create_query_executor(pool: &DbConnectionPool) -> QueryExecutor {
    QueryExecutor::new(pool.clone())
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    use super::*;
    use crate::db::SqlitePool;

    #[tokio::test]
    async fn sqlite_pool_creates_sqlite_executor_success() {
        let sqlite_pool = SqlitePool::new(
            SqlitePoolOptions::new().connect_lazy_with(SqliteConnectOptions::new().in_memory(true)),
        );
        let connection_pool = DbConnectionPool::Sqlite(sqlite_pool);

        let executor = create_query_executor(&connection_pool);

        executor
            .execute_write("CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .await
            .expect("expected sqlite-backed query executor to execute upsert");
    }
}
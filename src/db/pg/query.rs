use sqlx::AssertSqlSafe;
use std::future::Future;
use std::pin::Pin;

use crate::db::query::mapper::map_rows;
use crate::db::query::model::BackendQueryExecutor;
use crate::db::query::{DbQueryError, QueryRows};

use super::model::PostgresPool;

#[derive(Debug, Clone)]
pub struct PostgresQueryExecutor {
    pool: PostgresPool,
}

impl PostgresQueryExecutor {
    pub fn new(pool: PostgresPool) -> Self {
        Self { pool }
    }
}

impl BackendQueryExecutor for PostgresQueryExecutor {
    fn execute_backend_write<'a>(
        &'a self,
        sql: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, DbQueryError>> + Send + 'a>> {
        Box::pin(async move { execute_backend_write_inner(self.pool.inner(), sql).await })
    }

    fn execute_backend_read<'a>(
        &'a self,
        sql: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<QueryRows, DbQueryError>> + Send + 'a>> {
        Box::pin(async move { execute_backend_read_inner(self.pool.inner(), sql).await })
    }
}

async fn execute_backend_write_inner(
    pool: &sqlx::PgPool,
    sql: &str,
) -> Result<u64, DbQueryError> {
    let result = sqlx::raw_sql(AssertSqlSafe(sql))
        .execute(pool)
        .await
        .map_err(DbQueryError::Execution)?;
    Ok(result.rows_affected())
}

async fn execute_backend_read_inner(
    pool: &sqlx::PgPool,
    sql: &str,
) -> Result<QueryRows, DbQueryError> {
    let rows = sqlx::raw_sql(AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
        .map_err(DbQueryError::Execution)?;
    Ok(map_rows(rows))
}

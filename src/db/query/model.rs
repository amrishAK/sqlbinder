use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::error::DbQueryError;

#[derive(Debug, Clone)]
pub struct QueryExecutor {
    pub(super) backend_executor: Arc<dyn BackendQueryExecutor>,
}

pub(crate) trait BackendQueryExecutor: Send + Sync + std::fmt::Debug {
    fn execute_backend_write<'a>(
        &'a self,
        sql: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, DbQueryError>> + Send + 'a>>;

    fn execute_backend_read<'a>(
        &'a self,
        sql: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<QueryRows, DbQueryError>> + Send + 'a>>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryResult {
    AffectedRows(u64),
    Rows(QueryRows),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbValue {
    Null,
    String(String),
    Int64(i64),
    Float64(f64),
    Bool(bool),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRows {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<DbValue>>,
}

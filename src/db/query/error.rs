use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbQueryError {
    #[error("query execution failed: {0}")]
    Execution(#[source] sqlx::Error),
}
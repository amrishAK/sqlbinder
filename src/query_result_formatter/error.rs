use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryResultFormatterError {
    #[error("failed to serialize query result to json: {0}")]
    Serialization(#[source] serde_json::Error),
}
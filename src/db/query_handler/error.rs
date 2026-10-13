use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryHandlerError {
    #[error("failed to read query file '{path}': {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("query file is empty")]
    EmptyQuery,
    #[error("query contains an unterminated {0}")]
    UnterminatedSqlToken(&'static str),
    #[error("numbered placeholders are unsupported; use named placeholders such as $user_id")]
    NumberedPlaceholder,
    #[error("failed to tokenize SQL: {0}")]
    Tokenize(#[from] sqlparser::tokenizer::TokenizerError),
    #[error("failed to parse SQL: {0}")]
    Parse(#[from] sqlparser::parser::ParserError),
    #[error("expected exactly one SQL statement, found {0}")]
    StatementCount(usize),
    #[error("only one optional trailing semicolon is allowed")]
    StatementSeparator,
    #[error("unsupported SQL construct: {0}")]
    UnsupportedSql(&'static str),
    #[error("invalid {kind} name '{name}'; expected [a-z][a-z0-9_]* without quotes")]
    InvalidName { kind: &'static str, name: String },
    #[error("unsupported input placeholder '{0}'; use $name")]
    UnsupportedPlaceholder(String),
    #[error("every final output must have an explicit AS alias")]
    MissingOutputAlias,
    #[error("duplicate output alias '{0}'")]
    DuplicateOutputAlias(String),
    #[error("wildcard output projections are forbidden")]
    WildcardProjection,
    #[error("query is missing parameter values: {0:?}")]
    MissingParameters(Vec<String>),
    #[error("query received unused parameter values: {0:?}")]
    UnexpectedParameters(Vec<String>),
}

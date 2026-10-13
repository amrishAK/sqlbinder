use crate::db::DbValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryBackend {
    Postgres,
    Sqlite,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundQuery {
    pub sql: String,
    pub parameters: Vec<DbValue>,
}

#[derive(Debug)]
pub(super) struct ParsedQuery {
    pub(super) sql: String,
    pub(super) placeholders: Vec<Placeholder>,
    pub(super) statements: Vec<sqlparser::ast::Statement>,
    pub(super) tokens: Vec<sqlparser::tokenizer::TokenWithSpan>,
}

#[derive(Debug)]
pub(super) struct Placeholder {
    pub(super) name: String,
    pub(super) start: usize,
    pub(super) end: usize,
}

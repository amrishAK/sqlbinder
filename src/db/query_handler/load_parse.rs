use std::fs;
use std::path::Path;

use sqlparser::dialect::{Dialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Location, Token, Tokenizer};

use super::error::QueryHandlerError;
use super::model::{ParsedQuery, Placeholder, QueryBackend};

pub(super) fn load_and_parse(
    path: &Path,
    backend: QueryBackend,
) -> Result<ParsedQuery, QueryHandlerError> {
    let sql = fs::read_to_string(path).map_err(|source| QueryHandlerError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    parse_query(sql, backend)
}

pub(super) fn parse_query(
    sql: String,
    backend: QueryBackend,
) -> Result<ParsedQuery, QueryHandlerError> {
    if sql.trim().is_empty() {
        return Err(QueryHandlerError::EmptyQuery);
    }

    let dialect: &dyn Dialect = match backend {
        QueryBackend::Postgres => &PostgreSqlDialect {},
        QueryBackend::Sqlite => &SQLiteDialect {},
    };
    let tokens = Tokenizer::new(dialect, &sql).tokenize_with_location()?;
    let statements = Parser::new(dialect)
        .with_tokens_with_locations(tokens.clone())
        .parse_statements()?;

    let mut inputs = tokens
        .iter()
        .filter_map(|token| match &token.token {
            Token::Placeholder(value) => {
                value.strip_prefix('$').map(|name| (token.span.start, name))
            }
            _ => None,
        })
        .peekable();
    let mut placeholders = Vec::new();
    let mut location = Location { line: 1, column: 1 };
    // Token spans count characters; the binder needs byte offsets into the original SQL.
    for (offset, character) in sql.char_indices() {
        if inputs.peek().is_some_and(|(start, _)| *start == location) {
            let (_, name) = inputs.next().expect("input token was just checked");
            placeholders.push(Placeholder {
                name: name.to_owned(),
                start: offset,
                end: offset + 1 + name.len(),
            });
        }
        if character == '\n' {
            location.line += 1;
            location.column = 1;
        } else {
            location.column += 1;
        }
    }

    Ok(ParsedQuery {
        sql,
        placeholders,
        statements,
        tokens,
    })
}

use std::collections::{BTreeSet, HashMap};
use std::ops::ControlFlow;

use sqlparser::ast::{Query, SelectItem, SetExpr, Statement, Value, Visit, Visitor};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Location, Token};

use crate::db::DbValue;

use super::error::QueryHandlerError;
use super::model::ParsedQuery;

pub(super) fn validate_contract(query: &ParsedQuery) -> Result<(), QueryHandlerError> {
    if query.statements.len() != 1 {
        return Err(QueryHandlerError::StatementCount(query.statements.len()));
    }
    let significant = query
        .tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    let separators = significant
        .iter()
        .filter(|token| token.token == Token::SemiColon)
        .count();
    if separators > 1
        || (separators == 1
            && significant
                .last()
                .is_some_and(|token| token.token != Token::SemiColon))
    {
        return Err(QueryHandlerError::StatementSeparator);
    }
    for input in &query.placeholders {
        if input
            .name
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_digit)
        {
            return Err(QueryHandlerError::NumberedPlaceholder);
        }
        validate_name("input", &input.name)?;
    }
    let explicit_aliases = significant
        .windows(2)
        .filter_map(|pair| {
            matches!(&pair[0].token, Token::Word(word) if word.keyword == Keyword::AS)
                .then_some(pair[1].span.start)
        })
        .collect::<BTreeSet<_>>();
    let statement = &query.statements[0];
    if let ControlFlow::Break(error) = statement.visit(&mut ContractVisitor) {
        return Err(error);
    }
    validate_final_statement(statement, &explicit_aliases)
}

fn validate_name(kind: &'static str, name: &str) -> Result<(), QueryHandlerError> {
    let mut bytes = name.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(QueryHandlerError::InvalidName {
            kind,
            name: name.to_owned(),
        });
    }
    Ok(())
}

fn validate_projection(
    projection: &[SelectItem],
    explicit_aliases: Option<&BTreeSet<Location>>,
) -> Result<(), QueryHandlerError> {
    let mut names = BTreeSet::new();
    for item in projection {
        if matches!(
            item,
            SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(..)
        ) {
            return Err(QueryHandlerError::WildcardProjection);
        }
        if let Some(explicit_aliases) = explicit_aliases {
            let SelectItem::ExprWithAlias { alias, .. } = item else {
                return Err(QueryHandlerError::MissingOutputAlias);
            };
            if alias.quote_style.is_some() {
                return Err(QueryHandlerError::InvalidName {
                    kind: "output",
                    name: alias.to_string(),
                });
            }
            validate_name("output", &alias.value)?;
            if !explicit_aliases.contains(&alias.span.start) {
                return Err(QueryHandlerError::MissingOutputAlias);
            }
            if !names.insert(alias.value.as_str()) {
                return Err(QueryHandlerError::DuplicateOutputAlias(alias.value.clone()));
            }
        }
    }
    Ok(())
}

fn validate_final_statement(
    statement: &Statement,
    aliases: &BTreeSet<Location>,
) -> Result<(), QueryHandlerError> {
    match statement {
        Statement::Query(query) => validate_final_body(&query.body, aliases),
        Statement::Insert(insert) => validate_returning(insert.returning.as_deref(), Some(aliases)),
        Statement::Update { returning, .. } => {
            validate_returning(returning.as_deref(), Some(aliases))
        }
        Statement::Delete(delete) => validate_returning(delete.returning.as_deref(), Some(aliases)),
        _ => Err(QueryHandlerError::UnsupportedSql("statement type")),
    }
}

fn validate_final_body(
    body: &SetExpr,
    aliases: &BTreeSet<Location>,
) -> Result<(), QueryHandlerError> {
    match body {
        SetExpr::Select(select) => validate_projection(&select.projection, Some(aliases)),
        SetExpr::Query(query) => validate_final_body(&query.body, aliases),
        SetExpr::Insert(statement) | SetExpr::Update(statement) | SetExpr::Delete(statement) => {
            validate_final_statement(statement, aliases)
        }
        _ => Err(QueryHandlerError::UnsupportedSql("final query body")),
    }
}

fn validate_returning(
    projection: Option<&[SelectItem]>,
    aliases: Option<&BTreeSet<Location>>,
) -> Result<(), QueryHandlerError> {
    if let Some(projection) = projection {
        validate_projection(projection, aliases)?;
    }
    Ok(())
}

struct ContractVisitor;

impl Visitor for ContractVisitor {
    type Break = QueryHandlerError;

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        let result = match query.body.as_ref() {
            SetExpr::Select(select) => {
                if select.into.is_some() {
                    Err(QueryHandlerError::UnsupportedSql("SELECT INTO"))
                } else {
                    validate_projection(&select.projection, None)
                }
            }
            SetExpr::SetOperation { .. } => Err(QueryHandlerError::UnsupportedSql("set operation")),
            SetExpr::Query(_)
            | SetExpr::Values(_)
            | SetExpr::Insert(_)
            | SetExpr::Update(_)
            | SetExpr::Delete(_) => Ok(()),
            _ => Err(QueryHandlerError::UnsupportedSql("query body")),
        };
        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(error),
        }
    }

    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<Self::Break> {
        let result = match statement {
            Statement::Query(_) => Ok(()),
            Statement::Insert(insert) => validate_returning(insert.returning.as_deref(), None),
            Statement::Update { returning, .. } => validate_returning(returning.as_deref(), None),
            Statement::Delete(delete) => validate_returning(delete.returning.as_deref(), None),
            _ => Err(QueryHandlerError::UnsupportedSql("statement type")),
        };
        match result {
            Ok(()) => ControlFlow::Continue(()),
            Err(error) => ControlFlow::Break(error),
        }
    }

    fn pre_visit_value(&mut self, value: &Value) -> ControlFlow<Self::Break> {
        if let Value::Placeholder(value) = value
            && !value.starts_with('$')
        {
            return ControlFlow::Break(QueryHandlerError::UnsupportedPlaceholder(value.clone()));
        }
        ControlFlow::Continue(())
    }
}

pub(super) fn validate_parameters(
    query: &ParsedQuery,
    parameters: &HashMap<String, DbValue>,
) -> Result<(), QueryHandlerError> {
    let expected = query
        .placeholders
        .iter()
        .map(|placeholder| placeholder.name.as_str())
        .collect::<BTreeSet<_>>();
    let provided = parameters
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    let missing = expected
        .difference(&provided)
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(QueryHandlerError::MissingParameters(missing));
    }

    let unexpected = provided
        .difference(&expected)
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    if !unexpected.is_empty() {
        return Err(QueryHandlerError::UnexpectedParameters(unexpected));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::QueryBackend;
    use crate::db::query_handler::load_parse::parse_query;

    fn check(sql: &str, backend: QueryBackend) -> Result<(), QueryHandlerError> {
        validate_contract(&parse_query(sql.to_owned(), backend)?)
    }

    #[test]
    fn accepts_explicit_outputs_aggregates_and_internal_unaliased_columns() {
        let queries = [
            "SELECT id AS user_id, price * quantity AS total_price FROM users",
            "SELECT COUNT(*) AS item_count FROM users",
            "WITH source AS (SELECT id FROM users) SELECT id AS user_id FROM source",
            "SELECT x.id AS user_id FROM (SELECT id FROM users) AS x",
            "SELECT (SELECT MAX(id) FROM users) AS maximum_id",
            "SELECT '$bad; * AS fake' AS text_value /* SELECT *; */;",
            "SELECT $user_id AS user_id -- trailing comment",
        ];
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for sql in queries {
                check(sql, backend).unwrap_or_else(|error| panic!("{backend:?}: {sql}: {error}"));
            }
        }
    }

    #[test]
    fn rejects_wildcards_at_any_projection_depth() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for sql in [
                "SELECT * FROM users",
                "SELECT u.* FROM users AS u",
                "WITH source AS (SELECT * FROM users) SELECT id AS user_id FROM source",
                "SELECT x.id AS user_id FROM (SELECT u.* FROM users AS u) AS x",
                "SELECT (SELECT * FROM users) AS value",
                "INSERT INTO users SELECT * FROM source",
                "DELETE FROM users RETURNING *",
            ] {
                assert!(
                    matches!(
                        check(sql, backend),
                        Err(QueryHandlerError::WildcardProjection)
                    ),
                    "{backend:?}: {sql}"
                );
            }
        }
    }

    #[test]
    fn requires_explicit_as_on_final_outputs() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for sql in [
                "SELECT id FROM users",
                "SELECT id user_id FROM users",
                "SELECT CAST(id AS INTEGER) user_id FROM users",
                "SELECT id AS user_id, name FROM users",
                "UPDATE users SET name = $name RETURNING id",
            ] {
                assert!(
                    matches!(
                        check(sql, backend),
                        Err(QueryHandlerError::MissingOutputAlias)
                    ),
                    "{backend:?}: {sql}"
                );
            }
            check("SELECT id AS /* comment */ user_id FROM users", backend)
                .expect("comments between AS and alias are valid");
        }
    }

    #[test]
    fn rejects_duplicate_final_output_names() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for sql in [
                "SELECT id AS value, name AS value FROM users",
                "DELETE FROM users RETURNING id AS value, name AS value",
            ] {
                assert!(matches!(check(sql, backend),
                    Err(QueryHandlerError::DuplicateOutputAlias(name)) if name == "value"));
            }
        }
    }

    #[test]
    fn enforces_contract_name_grammar() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for name in ["User", "_user", "é", "userName"] {
                assert!(
                    matches!(
                        check(&format!("SELECT ${name} AS value"), backend),
                        Err(QueryHandlerError::InvalidName { kind: "input", .. })
                    ),
                    "{backend:?}: {name}"
                );
                assert!(
                    matches!(
                        check(&format!("SELECT 1 AS {name}"), backend),
                        Err(QueryHandlerError::InvalidName { kind: "output", .. })
                    ),
                    "{backend:?}: {name}"
                );
            }
            assert!(matches!(
                check("SELECT 1 AS \"value\"", backend),
                Err(QueryHandlerError::InvalidName { kind: "output", .. })
            ));
            check("SELECT $user_id2 AS value_2", backend).expect("valid names must pass");
        }
    }

    #[test]
    fn rejects_noncanonical_input_forms() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for input in ["$1", "$", "?", "?1", ":name", "$9name"] {
                assert!(
                    check(&format!("SELECT {input} AS value"), backend).is_err(),
                    "{backend:?}: {input}"
                );
            }
        }
        for input in ["?", "?1", ":name", "@name"] {
            assert!(
                matches!(
                    check(&format!("SELECT {input} AS value"), QueryBackend::Sqlite),
                    Err(QueryHandlerError::UnsupportedPlaceholder(_))
                ),
                "{input}"
            );
        }
    }

    #[test]
    fn requires_one_statement_and_only_a_trailing_semicolon() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            assert!(matches!(
                check("", backend),
                Err(QueryHandlerError::EmptyQuery)
            ));
            assert!(matches!(
                check("-- comment only", backend),
                Err(QueryHandlerError::StatementCount(0))
            ));
            assert!(matches!(
                check("SELECT 1 AS one; SELECT 2 AS two", backend),
                Err(QueryHandlerError::StatementCount(2))
            ));
            for sql in ["; SELECT 1 AS one", "SELECT 1 AS one;;"] {
                assert!(matches!(
                    check(sql, backend),
                    Err(QueryHandlerError::StatementSeparator)
                ));
            }
            check("SELECT ';' AS separator; -- comment\n", backend)
                .expect("literal semicolon and trailing comment are valid");
        }
    }

    #[test]
    fn accepts_writes_and_validates_returning() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for sql in [
                "INSERT INTO users (name) VALUES ($name)",
                "UPDATE users SET name = $name WHERE id = $user_id",
                "DELETE FROM users WHERE id = $user_id",
                "INSERT INTO users (name) VALUES ($name) RETURNING id AS user_id",
                "UPDATE users SET name = $name RETURNING id AS user_id",
                "DELETE FROM users RETURNING id AS user_id",
                "WITH source AS (SELECT id FROM users) DELETE FROM users \
                 WHERE id IN (SELECT id FROM source) RETURNING id AS user_id",
            ] {
                check(sql, backend).unwrap_or_else(|error| panic!("{backend:?}: {sql}: {error}"));
            }
        }
    }

    #[test]
    fn rejects_unsupported_statements_and_nested_set_operations() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            for sql in [
                "CREATE TABLE users (id INTEGER)",
                "DROP TABLE users",
                "SELECT id AS user_id FROM users UNION SELECT id FROM users",
                "SELECT id AS user_id FROM users INTERSECT SELECT id FROM users",
                "WITH source AS (SELECT id FROM users EXCEPT SELECT id FROM users) \
                 SELECT id AS user_id FROM source",
                "SELECT x.id AS user_id FROM (SELECT id FROM users UNION SELECT id FROM users) AS x",
            ] {
                assert!(
                    matches!(
                        check(sql, backend),
                        Err(QueryHandlerError::UnsupportedSql(_))
                    ),
                    "{backend:?}: {sql}"
                );
            }
            assert!(matches!(
                check(
                    "SELECT id AS user_id INTO copied_users FROM users",
                    QueryBackend::Postgres
                ),
                Err(QueryHandlerError::UnsupportedSql("SELECT INTO"))
            ));
        }
    }

    #[test]
    fn postgres_preserves_dollar_quotes_casts_json_operators_and_array_inputs() {
        for sql in [
            "SELECT $é$ body $ignored $é$ AS text_value, $value::INTEGER AS value",
            "SELECT $$ body $ignored $$ AS text_value, $value AS value",
            "SELECT E'a\\'$ignored' AS text_value, $value AS value",
            "SELECT metadata ? 'key' AS has_key FROM users",
            "SELECT @amount AS absolute_amount FROM users",
            "SELECT array_column[$index] AS element FROM users",
            "SELECT foo$bar AS value FROM users",
            "SELECT $value AS value /* outer /* $ignored */ comment */",
        ] {
            check(sql, QueryBackend::Postgres).unwrap_or_else(|error| panic!("{sql}: {error}"));
        }
    }

    #[test]
    fn rejects_invalid_sql_and_unterminated_tokens() {
        for backend in [QueryBackend::Postgres, QueryBackend::Sqlite] {
            assert!(matches!(
                check("SELECT FROM", backend),
                Err(QueryHandlerError::Parse(_))
            ));
            for sql in ["SELECT 'unterminated", "SELECT 1 AS value /* unclosed"] {
                assert!(matches!(
                    check(sql, backend),
                    Err(QueryHandlerError::Tokenize(_))
                ));
            }
        }
    }
}

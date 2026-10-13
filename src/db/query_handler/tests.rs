use std::collections::HashMap;
use std::path::PathBuf;

use crate::db::DbValue;

use super::*;

fn query_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test-utils")
        .join("queries")
        .join(name)
}

#[test]
fn prepare_query_rewrites_named_parameter_for_postgres() {
    let parameters = HashMap::from([("user_id".to_owned(), DbValue::Int64(7))]);

    let query = prepare_query(
        query_fixture("get_user_by_id.sql"),
        QueryBackend::Postgres,
        &parameters,
    )
    .expect("query should prepare");

    assert!(query.sql.contains("WHERE id = $1"));
    assert_eq!(query.parameters, vec![DbValue::Int64(7)]);
}

#[test]
fn prepare_query_rewrites_named_parameter_for_sqlite() {
    let parameters = HashMap::from([(
        "dataset_name".to_owned(),
        DbValue::String("sample-images".to_owned()),
    )]);

    let query = prepare_query(
        query_fixture("list_dataset_items.sql"),
        QueryBackend::Sqlite,
        &parameters,
    )
    .expect("query should prepare");

    assert!(query.sql.contains("WHERE dataset.name = ?1"));
    assert_eq!(
        query.parameters,
        vec![DbValue::String("sample-images".to_owned())]
    );
}

#[test]
fn prepare_query_rejects_missing_parameter() {
    let error = prepare_query(
        query_fixture("get_user_by_id.sql"),
        QueryBackend::Postgres,
        &HashMap::new(),
    )
    .expect_err("missing query parameters should be rejected");

    assert!(
        matches!(error, QueryHandlerError::MissingParameters(names) if names == vec!["user_id"])
    );
}

#[test]
fn prepare_query_rejects_unused_parameter() {
    let parameters = HashMap::from([
        ("user_id".to_owned(), DbValue::Int64(7)),
        ("unused".to_owned(), DbValue::Null),
    ]);
    let error = prepare_query(
        query_fixture("get_user_by_id.sql"),
        QueryBackend::Sqlite,
        &parameters,
    )
    .expect_err("unused parameters must fail");
    assert!(
        matches!(error, QueryHandlerError::UnexpectedParameters(names)
        if names == vec!["unused"])
    );
}

#[test]
fn prepare_query_reports_read_errors_with_path() {
    let path = query_fixture("does_not_exist.sql");
    let error = prepare_query(&path, QueryBackend::Sqlite, &HashMap::new())
        .expect_err("missing file must fail");
    assert!(
        matches!(error, QueryHandlerError::ReadFile { path: reported, .. }
        if reported == path)
    );
}

#[test]
fn prepare_query_validates_contract_before_supplied_parameters() {
    let error = prepare_query(
        query_fixture("invalid_output.sql"),
        QueryBackend::Sqlite,
        &HashMap::new(),
    )
    .expect_err("output contract must be checked before missing inputs");
    assert!(matches!(error, QueryHandlerError::MissingOutputAlias));
}

#[test]
fn binding_preserves_sql_and_orders_distinct_inputs_by_first_use() {
    let sql = "SELECT 'é; $ignored' AS label, $second AS second, \
                   $first AS first, $second AS repeated -- $comment\n";
    let parameters = HashMap::from([
        ("first".to_owned(), DbValue::String("O'Reilly".to_owned())),
        ("second".to_owned(), DbValue::Int64(7)),
    ]);
    for (backend, prefix) in [(QueryBackend::Postgres, "$"), (QueryBackend::Sqlite, "?")] {
        let parsed = load_parse::parse_query(sql.to_owned(), backend).expect("query must parse");
        validator::validate_contract(&parsed).expect("contract must be valid");
        validator::validate_parameters(&parsed, &parameters).expect("inputs must match");
        let bound = binder::bind_query(&parsed, backend, &parameters);
        assert_eq!(
            bound.sql,
            format!(
                "SELECT 'é; $ignored' AS label, {prefix}1 AS second, \
                 {prefix}2 AS first, {prefix}1 AS repeated -- $comment\n"
            )
        );
        assert_eq!(
            bound.parameters,
            vec![DbValue::Int64(7), DbValue::String("O'Reilly".to_owned()),]
        );
    }
}

#[test]
fn binding_preserves_multiline_literals_quotes_and_crlf_byte_offsets() {
    for (backend, literal, identifier) in [
        (
            QueryBackend::Postgres,
            "$é$ text $ignored $é$",
            "\"$identifier\"",
        ),
        (QueryBackend::Sqlite, "'é '' $ignored'", "[$identifier]"),
    ] {
        let sql = format!(
            "-- é $comment\r\nSELECT {literal} AS text_value,\r\n\
                 {identifier} AS identifier_value, $value AS value;"
        );
        let parsed = load_parse::parse_query(sql.clone(), backend).expect("SQL must parse");
        validator::validate_contract(&parsed).expect("contract must be valid");
        let parameters = HashMap::from([("value".to_owned(), DbValue::Bool(true))]);
        validator::validate_parameters(&parsed, &parameters).expect("one input must match");
        let bound = binder::bind_query(&parsed, backend, &parameters);
        let replacement = match backend {
            QueryBackend::Postgres => "$1",
            QueryBackend::Sqlite => "?1",
        };
        assert_eq!(bound.sql, sql.replace("$value", replacement));
        assert_eq!(bound.parameters, vec![DbValue::Bool(true)]);
    }
}

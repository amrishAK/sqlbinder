use std::collections::HashMap;
use std::path::Path;

use crate::db::DbValue;

use super::binder::bind_query;
use super::error::QueryHandlerError;
use super::load_parse::load_and_parse;
use super::model::{BoundQuery, QueryBackend};
use super::validator::{validate_contract, validate_parameters};

/// Loads and parses one SQL statement, validates its contract and inputs, then binds it.
///
/// Values are returned separately from SQL; this function does not execute the query.
///
/// # Errors
///
/// Returns an error for unreadable files, invalid SQL, contract violations, or
/// missing or unused input values.
pub fn prepare_query(
    query_file: impl AsRef<Path>,
    backend: QueryBackend,
    parameters: &HashMap<String, DbValue>,
) -> Result<BoundQuery, QueryHandlerError> {
    let parsed_query = load_and_parse(query_file.as_ref(), backend)?;
    validate_contract(&parsed_query)?;
    validate_parameters(&parsed_query, parameters)?;
    Ok(bind_query(&parsed_query, backend, parameters))
}

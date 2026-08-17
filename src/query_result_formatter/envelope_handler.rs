use super::error::QueryResultFormatterError;
use super::model::{QueryResultEnvelope, QueryResultItem, QueryResultValue};
use crate::db::query::{DbValue, QueryRows};
use std::collections::BTreeMap;

pub fn to_envelope_string(result: &QueryRows) -> Result<String, QueryResultFormatterError> {
    let envelope = to_envelope(result)?;
    serde_json::to_string(&envelope).map_err(QueryResultFormatterError::Serialization)
}

pub fn to_envelope(result: &QueryRows) -> Result<QueryResultEnvelope, QueryResultFormatterError> {
    Ok(QueryResultEnvelope {
        count: result.rows.len(),
        items: result
            .rows
            .iter()
            .map(|row| QueryResultItem {
                values: result
                    .columns
                    .iter()
                    .cloned()
                    .zip(row.iter().cloned())
                    .map(|(column, value)| (column, to_query_result_value(value)))
                    .collect::<BTreeMap<_, _>>(),
            })
            .collect(),
    })
}

fn to_query_result_value(value: DbValue) -> QueryResultValue {
    match value {
        DbValue::Null => QueryResultValue::Null,
        DbValue::String(value) => QueryResultValue::String(value),
        DbValue::Int64(value) => QueryResultValue::Integer(value),
        DbValue::Float64(value) => QueryResultValue::Float(value),
        DbValue::Bool(value) => QueryResultValue::Bool(value),
        DbValue::Bytes(value) => QueryResultValue::Bytes(value),
    }
}

#[cfg(test)]
mod tests {
    use super::{to_envelope, to_envelope_string};
    use crate::db::query::{DbValue, QueryRows};

    #[test]
    fn to_envelope_string_serializes_rows_success() {
        let result = QueryRows {
            columns: vec!["id".to_owned(), "name".to_owned(), "payload".to_owned()],
            rows: vec![vec![
                DbValue::Int64(1),
                DbValue::String("alice".to_owned()),
                DbValue::Bytes(vec![1, 2, 3]),
            ]],
        };

        let json = to_envelope_string(&result).expect("expected json serialization to succeed");

        assert_eq!(
            json,
            r#"{"count":1,"items":[{"id":1,"name":"alice","payload":[1,2,3]}]}"#
        );
    }

    #[test]
    fn to_envelope_string_serializes_empty_rows_success() {
        let result = QueryRows {
            columns: vec![],
            rows: vec![],
        };

        let json = to_envelope_string(&result).expect("expected json serialization to succeed");

        assert_eq!(json, r#"{"count":0,"items":[]}"#);
    }

    #[test]
    fn to_envelope_builds_expected_shape() {
        let result = QueryRows {
            columns: vec!["id".to_owned()],
            rows: vec![vec![DbValue::Int64(1)]],
        };

        let envelope = to_envelope(&result).expect("expected envelope build to succeed");

        assert_eq!(envelope.count, 1);
        assert_eq!(envelope.items.len(), 1);
    }
}
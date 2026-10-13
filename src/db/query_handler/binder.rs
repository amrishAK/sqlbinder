use std::collections::HashMap;

use crate::db::DbValue;

use super::model::{BoundQuery, ParsedQuery, QueryBackend};

pub(super) fn bind_query(
    query: &ParsedQuery,
    backend: QueryBackend,
    values: &HashMap<String, DbValue>,
) -> BoundQuery {
    let mut sql = String::with_capacity(query.sql.len());
    let mut parameters = Vec::new();
    let mut parameter_indexes = HashMap::new();
    let mut copied_until = 0;

    for placeholder in &query.placeholders {
        sql.push_str(&query.sql[copied_until..placeholder.start]);
        let index = *parameter_indexes
            .entry(placeholder.name.as_str())
            .or_insert_with(|| {
                parameters.push(values[&placeholder.name].clone());
                parameters.len()
            });
        match backend {
            QueryBackend::Postgres => sql.push_str(&format!("${index}")),
            QueryBackend::Sqlite => sql.push_str(&format!("?{index}")),
        }
        copied_until = placeholder.end;
    }
    sql.push_str(&query.sql[copied_until..]);

    BoundQuery { sql, parameters }
}

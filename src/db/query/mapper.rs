use sqlx::{Column, Row};

use super::model::{DbValue, QueryRows};

pub(crate) fn map_rows<R>(rows: Vec<R>) -> QueryRows
where
    R: Row,
    usize: sqlx::ColumnIndex<R>,
    for<'r> Option<String>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<i64>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<f64>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<bool>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<Vec<u8>>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    if rows.is_empty() {
        return QueryRows {
            columns: vec![],
            rows: vec![],
        };
    }

    let columns = rows[0]
        .columns()
        .iter()
        .map(|col| col.name().to_owned())
        .collect::<Vec<_>>();

    let values = rows
        .iter()
        .map(|row| {
            (0..columns.len())
                .map(|index| value_to_db_value(row, index))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    QueryRows {
        columns,
        rows: values,
    }
}

fn value_to_db_value<R>(row: &R, index: usize) -> DbValue
where
    R: Row,
    usize: sqlx::ColumnIndex<R>,
    for<'r> Option<String>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<i64>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<f64>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<bool>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    for<'r> Option<Vec<u8>>: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    if let Ok(value) = row.try_get::<Option<String>, _>(index) {
        return value.map(DbValue::String).unwrap_or(DbValue::Null);
    }
    if let Ok(value) = row.try_get::<Option<i64>, _>(index) {
        return value.map(DbValue::Int64).unwrap_or(DbValue::Null);
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(index) {
        return value.map(DbValue::Float64).unwrap_or(DbValue::Null);
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(index) {
        return value.map(DbValue::Bool).unwrap_or(DbValue::Null);
    }
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(index) {
        return value.map(DbValue::Bytes).unwrap_or(DbValue::Null);
    }
    DbValue::Null
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    use super::map_rows;
    use crate::db::DbValue;

    fn in_memory_pool() -> sqlx::SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_lazy_with(SqliteConnectOptions::new().in_memory(true))
    }

    #[tokio::test]
    async fn map_rows_returns_empty_result_for_empty_select_success() {
        let pool = in_memory_pool();

        let rows = sqlx::query("SELECT 1 WHERE 0")
            .fetch_all(&pool)
            .await
            .expect("expected select without results to succeed");

        let mapped = map_rows(rows);

        assert!(mapped.columns.is_empty());
        assert!(mapped.rows.is_empty());
    }

    #[tokio::test]
    async fn map_rows_preserves_column_names_and_db_value_variants_success() {
        let pool = in_memory_pool();

        sqlx::query(
            "CREATE TABLE sample(id INTEGER PRIMARY KEY, name TEXT, ratio REAL, is_active BOOLEAN, payload BLOB, note TEXT)",
        )
        .execute(&pool)
        .await
        .expect("expected create table to succeed");

        sqlx::query(
            "INSERT INTO sample(name, ratio, is_active, payload, note) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("alice")
        .bind(3.5f64)
        .bind(true)
        .bind(vec![1_u8, 2_u8, 3_u8])
        .bind::<Option<&str>>(None)
        .execute(&pool)
        .await
        .expect("expected insert query to succeed");

        let rows = sqlx::query(
            "SELECT id, name, ratio, is_active, payload, note FROM sample",
        )
        .fetch_all(&pool)
        .await
        .expect("expected row mapping query to succeed");

        let mapped = map_rows(rows);

        assert_eq!(
            mapped.columns,
            vec!["id", "name", "ratio", "is_active", "payload", "note"]
        );
        assert_eq!(
            mapped.rows,
            vec![vec![
                DbValue::Int64(1),
                DbValue::String("alice".to_owned()),
                DbValue::Float64(3.5),
                DbValue::Int64(1),
                DbValue::Bytes(vec![1_u8, 2_u8, 3_u8]),
                DbValue::Null,
            ]]
        );
    }
}

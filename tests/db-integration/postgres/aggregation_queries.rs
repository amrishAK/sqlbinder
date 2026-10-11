use crate::support::seeded_postgres_executor;
use sqlbinder::db::DbValue;

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn group_by_returns_user_status_counts() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT status, COUNT(*)
             FROM users
             GROUP BY status
             ORDER BY status",
        )
        .await
        .expect("status aggregation should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("active".to_owned()), DbValue::Int64(4)],
            vec![DbValue::String("disabled".to_owned()), DbValue::Int64(1)],
        ]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn numeric_filter_returns_float_scores() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT external_id, score::float8
             FROM dataset_items
             WHERE score >= 0.9
             ORDER BY score DESC",
        )
        .await
        .expect("numeric filter should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![
                DbValue::String("doc-001".to_owned()),
                DbValue::Float64(0.9921)
            ],
            vec![
                DbValue::String("img-001".to_owned()),
                DbValue::Float64(0.9812)
            ],
        ]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn having_clause_filters_grouped_rows() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT d.name, COUNT(di.id) AS item_count
             FROM datasets d
             JOIN dataset_items di ON di.dataset_id = d.id
             GROUP BY d.name
             HAVING COUNT(di.id) > 2
             ORDER BY d.name",
        )
        .await
        .expect("having aggregation should succeed");

    assert_eq!(rows.columns, ["name", "item_count"]);
    assert_eq!(
        rows.rows,
        vec![vec![
            DbValue::String("sample-images".to_owned()),
            DbValue::Int64(4),
        ]]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn aggregate_functions_return_min_max_and_sum() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read("SELECT MIN(age)::bigint, MAX(age)::bigint, SUM(age)::bigint FROM users")
        .await
        .expect("aggregate functions should succeed");

    assert_eq!(
        rows.rows,
        vec![vec![
            DbValue::Int64(26),
            DbValue::Int64(41),
            DbValue::Int64(168),
        ]]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn limit_and_offset_page_through_ordered_rows() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read("SELECT name FROM users ORDER BY name LIMIT 2 OFFSET 2")
        .await
        .expect("paged query should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("Charlie".to_owned())],
            vec![DbValue::String("Diana".to_owned())],
        ]
    );
}

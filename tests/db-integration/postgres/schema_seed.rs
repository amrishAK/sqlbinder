use crate::support::seeded_postgres_executor;
use sqlbinder::db::DbValue;

#[tokio::test]
async fn postgres_settings_create_executor_success() {
    let _executor = crate::support::postgres_executor();
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn seed_creates_expected_tables() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT table_name
             FROM information_schema.tables
             WHERE table_schema = 'public'
             ORDER BY table_name",
        )
        .await
        .expect("table listing should be readable");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("dataset_items".to_owned())],
            vec![DbValue::String("datasets".to_owned())],
            vec![DbValue::String("item_tags".to_owned())],
            vec![DbValue::String("team_members".to_owned())],
            vec![DbValue::String("teams".to_owned())],
            vec![DbValue::String("users".to_owned())],
        ]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn seed_creates_expected_table_counts() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT
                (SELECT COUNT(*) FROM users),
                (SELECT COUNT(*) FROM teams),
                (SELECT COUNT(*) FROM team_members),
                (SELECT COUNT(*) FROM datasets),
                (SELECT COUNT(*) FROM dataset_items),
                (SELECT COUNT(*) FROM item_tags)",
        )
        .await
        .expect("seeded table counts should be readable");

    assert_eq!(
        rows.rows,
        vec![vec![
            DbValue::Int64(5),
            DbValue::Int64(3),
            DbValue::Int64(7),
            DbValue::Int64(4),
            DbValue::Int64(10),
            DbValue::Int64(14),
        ]]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn read_returns_declared_column_names() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read("SELECT id, name, email, status FROM users ORDER BY id LIMIT 1")
        .await
        .expect("users projection should be readable");

    assert_eq!(rows.columns, ["id", "name", "email", "status"]);
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn empty_result_set_has_no_columns_and_no_rows() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read("SELECT id, name FROM users WHERE name = 'nobody'")
        .await
        .expect("empty select should succeed");

    assert!(rows.columns.is_empty());
    assert!(rows.rows.is_empty());
}

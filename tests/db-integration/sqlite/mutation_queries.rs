use crate::support::seeded_sqlite_executor;
use sqlbinder::db::DbValue;

#[tokio::test]
async fn insert_returns_affected_rows_and_is_readable() {
    let executor = seeded_sqlite_executor().await;

    let affected = executor
        .execute_write(
            "INSERT INTO users (name, email, age) VALUES ('Frank', 'frank@example.com', 45)",
        )
        .await
        .expect("user insert should succeed");
    assert_eq!(affected, 1);

    let rows = executor
        .execute_read("SELECT name, age FROM users WHERE email = 'frank@example.com'")
        .await
        .expect("inserted user should be readable");

    assert_eq!(
        rows.rows,
        vec![vec![
            DbValue::String("Frank".to_owned()),
            DbValue::Int64(45)
        ]]
    );
}

#[tokio::test]
async fn update_changes_matching_rows_only() {
    let executor = seeded_sqlite_executor().await;

    let affected = executor
        .execute_write("UPDATE users SET status = 'disabled' WHERE email = 'diana@example.com'")
        .await
        .expect("user update should succeed");
    assert_eq!(affected, 1);

    let rows = executor
        .execute_read("SELECT status, COUNT(*) FROM users GROUP BY status ORDER BY status")
        .await
        .expect("updated statuses should be readable");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("active".to_owned()), DbValue::Int64(3)],
            vec![DbValue::String("disabled".to_owned()), DbValue::Int64(2)],
        ]
    );
}

#[tokio::test]
async fn update_without_matches_reports_zero_affected_rows() {
    let executor = seeded_sqlite_executor().await;

    let affected = executor
        .execute_write("UPDATE users SET status = 'disabled' WHERE email = 'missing@example.com'")
        .await
        .expect("no-op update should succeed");

    assert_eq!(affected, 0);
}

#[tokio::test]
async fn delete_cascades_to_dependent_rows() {
    let executor = seeded_sqlite_executor().await;

    let affected = executor
        .execute_write("DELETE FROM datasets WHERE name = 'documents'")
        .await
        .expect("dataset deletion should succeed");
    assert_eq!(affected, 1);

    let rows = executor
        .execute_read(
            "SELECT COUNT(*) FROM dataset_items WHERE dataset_id = (SELECT id FROM datasets WHERE name = 'documents')",
        )
        .await
        .expect("cascade result should be readable");

    assert_eq!(rows.rows, vec![vec![DbValue::Int64(0)]]);
}

#[tokio::test]
async fn delete_cascades_from_items_to_tags() {
    let executor = seeded_sqlite_executor().await;

    executor
        .execute_write("DELETE FROM dataset_items WHERE external_id = 'img-001'")
        .await
        .expect("item deletion should succeed");

    let rows = executor
        .execute_read("SELECT COUNT(*) FROM item_tags")
        .await
        .expect("remaining tags should be readable");

    assert_eq!(rows.rows, vec![vec![DbValue::Int64(12)]]);
}

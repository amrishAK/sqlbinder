use crate::support::seeded_postgres_executor;
use sqlbinder::db::DbValue;

// The PostgreSQL fixture is shared across tests, so every mutation test works on
// rows it creates itself and removes them again.

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn insert_update_delete_roundtrip() {
    let executor = seeded_postgres_executor().await;

    executor
        .execute_write("DELETE FROM users WHERE email = 'mutation-roundtrip@example.com'")
        .await
        .expect("pre-cleanup should succeed");

    let affected = executor
        .execute_write(
            "INSERT INTO users (name, email, age) VALUES ('Roundtrip', 'mutation-roundtrip@example.com', 45)",
        )
        .await
        .expect("user insert should succeed");
    assert_eq!(affected, 1);

    let affected = executor
        .execute_write(
            "UPDATE users SET status = 'disabled' WHERE email = 'mutation-roundtrip@example.com'",
        )
        .await
        .expect("user update should succeed");
    assert_eq!(affected, 1);

    let rows = executor
        .execute_read(
            "SELECT name, status FROM users WHERE email = 'mutation-roundtrip@example.com'",
        )
        .await
        .expect("mutated user should be readable");
    assert_eq!(
        rows.rows,
        vec![vec![
            DbValue::String("Roundtrip".to_owned()),
            DbValue::String("disabled".to_owned()),
        ]]
    );

    let affected = executor
        .execute_write("DELETE FROM users WHERE email = 'mutation-roundtrip@example.com'")
        .await
        .expect("user delete should succeed");
    assert_eq!(affected, 1);
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn update_without_matches_reports_zero_affected_rows() {
    let executor = seeded_postgres_executor().await;

    let affected = executor
        .execute_write("UPDATE users SET status = 'disabled' WHERE email = 'missing@example.com'")
        .await
        .expect("no-op update should succeed");

    assert_eq!(affected, 0);
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn delete_cascades_to_dependent_rows() {
    let executor = seeded_postgres_executor().await;

    executor
        .execute_write("DELETE FROM datasets WHERE name = 'cascade-fixture'")
        .await
        .expect("pre-cleanup should succeed");

    executor
        .execute_write("INSERT INTO datasets (name) VALUES ('cascade-fixture')")
        .await
        .expect("dataset insert should succeed");
    executor
        .execute_write(
            "INSERT INTO dataset_items (dataset_id, external_id)
             SELECT id, 'cascade-item' FROM datasets WHERE name = 'cascade-fixture'",
        )
        .await
        .expect("item insert should succeed");

    executor
        .execute_write("DELETE FROM datasets WHERE name = 'cascade-fixture'")
        .await
        .expect("dataset deletion should succeed");

    let rows = executor
        .execute_read("SELECT COUNT(*) FROM dataset_items WHERE external_id = 'cascade-item'")
        .await
        .expect("cascade result should be readable");

    assert_eq!(rows.rows, vec![vec![DbValue::Int64(0)]]);
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn duplicate_unique_value_is_rejected() {
    let executor = seeded_postgres_executor().await;

    let result = executor
        .execute_write(
            "INSERT INTO users (name, email, age) VALUES ('Other', 'alice@example.com', 20)",
        )
        .await;

    assert!(
        result.is_err(),
        "unique email constraint should reject duplicates"
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn query_against_unknown_table_fails() {
    let executor = seeded_postgres_executor().await;

    let result = executor.execute_read("SELECT * FROM does_not_exist").await;

    assert!(result.is_err(), "unknown table should produce an error");
}

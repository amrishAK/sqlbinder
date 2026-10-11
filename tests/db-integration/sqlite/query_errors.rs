use crate::support::{seeded_sqlite_executor, sqlite_executor};

#[tokio::test]
async fn duplicate_unique_value_is_rejected() {
    let executor = seeded_sqlite_executor().await;

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
async fn foreign_key_violation_is_rejected() {
    let executor = seeded_sqlite_executor().await;

    let result = executor
        .execute_write("INSERT INTO team_members (team_id, user_id) VALUES (999, 1)")
        .await;

    assert!(
        result.is_err(),
        "foreign key constraint should reject unknown team ids"
    );
}

#[tokio::test]
async fn query_against_unknown_table_fails() {
    let executor = sqlite_executor();

    let result = executor.execute_read("SELECT * FROM does_not_exist").await;

    assert!(result.is_err(), "unknown table should produce an error");
}

#[tokio::test]
async fn syntactically_invalid_sql_fails() {
    let executor = sqlite_executor();

    let result = executor.execute_read("SELCT 1").await;

    assert!(result.is_err(), "invalid SQL should produce an error");
}

#[tokio::test]
async fn not_null_violation_is_rejected() {
    let executor = seeded_sqlite_executor().await;

    let result = executor
        .execute_write("INSERT INTO users (name, email, age) VALUES (NULL, 'x@example.com', 20)")
        .await;

    assert!(result.is_err(), "NOT NULL constraint should be enforced");
}

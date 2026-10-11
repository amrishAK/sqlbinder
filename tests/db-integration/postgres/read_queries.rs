use crate::support::seeded_postgres_executor;
use sqlbinder::db::DbValue;

// PostgreSQL-specific types (INTEGER, NUMERIC, JSONB) are cast explicitly so the
// backend-agnostic `DbValue` mapping stays predictable.

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn reads_seeded_users() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read("SELECT id, name, age::bigint, status FROM users ORDER BY id")
        .await
        .expect("seeded users should be readable");

    assert_eq!(rows.columns, ["id", "name", "age", "status"]);
    assert_eq!(rows.rows.len(), 5);
    assert_eq!(
        rows.rows[0],
        vec![
            DbValue::Int64(1),
            DbValue::String("Alice".to_owned()),
            DbValue::Int64(29),
            DbValue::String("active".to_owned()),
        ]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn join_returns_team_membership_rows() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT u.name, t.name, tm.role
             FROM team_members tm
             JOIN users u ON u.id = tm.user_id
             JOIN teams t ON t.id = tm.team_id
             WHERE t.name = 'platform'
             ORDER BY tm.user_id",
        )
        .await
        .expect("seeded relationship query should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![
                DbValue::String("Alice".to_owned()),
                DbValue::String("platform".to_owned()),
                DbValue::String("owner".to_owned()),
            ],
            vec![
                DbValue::String("Bob".to_owned()),
                DbValue::String("platform".to_owned()),
                DbValue::String("member".to_owned()),
            ],
            vec![
                DbValue::String("Eve".to_owned()),
                DbValue::String("platform".to_owned()),
                DbValue::String("member".to_owned()),
            ],
        ]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn null_columns_map_to_db_value_null() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT external_id, image_url
             FROM dataset_items
             WHERE label = 'bird' AND image_url IS NULL",
        )
        .await
        .expect("nullable item query should succeed");

    assert_eq!(
        rows.rows,
        vec![vec![DbValue::String("img-004".to_owned()), DbValue::Null]]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn jsonb_filter_matches_metadata_keys() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT external_id, metadata->>'format'
             FROM dataset_items
             WHERE metadata->>'format' = 'jpg'
             ORDER BY external_id",
        )
        .await
        .expect("jsonb filter should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![
                DbValue::String("img-001".to_owned()),
                DbValue::String("jpg".to_owned()),
            ],
            vec![
                DbValue::String("img-002".to_owned()),
                DbValue::String("jpg".to_owned()),
            ],
        ]
    );
}

#[tokio::test]
#[ignore = "requires the PostgreSQL fixture from tests/test-utils/postgres"]
async fn tag_lookup_preserves_requested_ordering() {
    let executor = seeded_postgres_executor().await;

    let rows = executor
        .execute_read(
            "SELECT di.external_id
             FROM item_tags it
             JOIN dataset_items di ON di.id = it.item_id
             WHERE it.tag = 'featured'
             ORDER BY di.id",
        )
        .await
        .expect("tag lookup should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("img-001".to_owned())],
            vec![DbValue::String("img-003".to_owned())],
            vec![DbValue::String("doc-001".to_owned())],
            vec![DbValue::String("evt-001".to_owned())],
        ]
    );
}

use crate::support::seeded_sqlite_executor;
use sqlbinder::db::DbValue;

#[tokio::test]
async fn join_returns_team_membership_rows() {
    let executor = seeded_sqlite_executor().await;

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
async fn left_join_keeps_datasets_without_matching_items() {
    let executor = seeded_sqlite_executor().await;

    let rows = executor
        .execute_read(
            "SELECT d.name, COUNT(di.id)
             FROM datasets d
             LEFT JOIN dataset_items di ON di.dataset_id = d.id AND di.label = 'invoice'
             GROUP BY d.name
             ORDER BY d.name",
        )
        .await
        .expect("left join aggregation should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("audit-log".to_owned()), DbValue::Int64(0)],
            vec![DbValue::String("documents".to_owned()), DbValue::Int64(1)],
            vec![DbValue::String("events".to_owned()), DbValue::Int64(0)],
            vec![
                DbValue::String("sample-images".to_owned()),
                DbValue::Int64(0)
            ],
        ]
    );
}

#[tokio::test]
async fn null_columns_map_to_db_value_null() {
    let executor = seeded_sqlite_executor().await;

    let rows = executor
        .execute_read(
            "SELECT external_id, image_url, metadata
             FROM dataset_items
             WHERE label = 'bird' AND image_url IS NULL",
        )
        .await
        .expect("nullable item query should succeed");

    assert_eq!(
        rows.rows,
        vec![vec![
            DbValue::String("img-004".to_owned()),
            DbValue::Null,
            DbValue::String("{\"notes\":\"missing url for null-path testing\"}".to_owned()),
        ]]
    );
}

#[tokio::test]
async fn like_filter_matches_text_metadata() {
    let executor = seeded_sqlite_executor().await;

    let rows = executor
        .execute_read(
            "SELECT external_id
             FROM dataset_items
             WHERE metadata LIKE '%\"format\":\"jpg\"%'
             ORDER BY external_id",
        )
        .await
        .expect("metadata filter should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("img-001".to_owned())],
            vec![DbValue::String("img-002".to_owned())],
        ]
    );
}

#[tokio::test]
async fn tag_lookup_preserves_requested_ordering() {
    let executor = seeded_sqlite_executor().await;

    let rows = executor
        .execute_read(
            "SELECT di.external_id, it.tag
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
            vec![
                DbValue::String("img-001".to_owned()),
                DbValue::String("featured".to_owned()),
            ],
            vec![
                DbValue::String("img-003".to_owned()),
                DbValue::String("featured".to_owned()),
            ],
            vec![
                DbValue::String("doc-001".to_owned()),
                DbValue::String("featured".to_owned()),
            ],
            vec![
                DbValue::String("evt-001".to_owned()),
                DbValue::String("featured".to_owned()),
            ],
        ]
    );
}

#[tokio::test]
async fn subquery_and_in_filter_return_expected_users() {
    let executor = seeded_sqlite_executor().await;

    let rows = executor
        .execute_read(
            "SELECT name
             FROM users
             WHERE id IN (SELECT owner_user_id FROM teams WHERE owner_user_id IS NOT NULL)
             ORDER BY name",
        )
        .await
        .expect("subquery filter should succeed");

    assert_eq!(
        rows.rows,
        vec![
            vec![DbValue::String("Alice".to_owned())],
            vec![DbValue::String("Bob".to_owned())],
            vec![DbValue::String("Diana".to_owned())],
        ]
    );
}

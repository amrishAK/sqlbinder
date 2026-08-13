use std::path::PathBuf;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::context_container::app_settings::model::{SqliteInMemorySettings, SqliteSettings};
use crate::db::connection_pool::error::DbConnectionError;

use super::model::SqlitePool;

pub(crate) fn get_sqlite_pool(database_settings: &SqliteSettings) -> Result<SqlitePool, DbConnectionError> {
    let options = create_connection_options(database_settings)?;
    let pool = SqlitePoolOptions::new()
        .max_connections(database_settings.max_connections)
        .acquire_timeout(Duration::from_secs(database_settings.connect_timeout_secs))
        .connect_lazy_with(options);

    Ok(SqlitePool::new(pool))
}

pub(crate) fn get_sqlite_in_memory_pool(
    database_settings: &SqliteInMemorySettings,
) -> Result<SqlitePool, DbConnectionError> {
    let options = SqliteConnectOptions::new().in_memory(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(database_settings.max_connections)
        .acquire_timeout(Duration::from_secs(database_settings.connect_timeout_secs))
        .connect_lazy_with(options);

    Ok(SqlitePool::new(pool))
}

fn create_connection_options(database_settings: &SqliteSettings) -> Result<SqliteConnectOptions, DbConnectionError> {
    let db_path = resolve_sqlite_database_path(database_settings)?;
    Ok(SqliteConnectOptions::new().filename(db_path))
}

fn resolve_sqlite_database_path(database_settings: &SqliteSettings) -> Result<PathBuf, DbConnectionError> {
    let directory = database_settings
        .directory
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| DbConnectionError::InvalidConfiguration {
            field: "database.directory",
            message: "must be provided for file-based sqlite".to_owned(),
        })?;

    let filename = database_settings
        .filename
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| DbConnectionError::InvalidConfiguration {
            field: "database.filename",
            message: "must be provided for file-based sqlite".to_owned(),
        })?;

    let mut db_path = PathBuf::from(directory);
    db_path.push(filename);
    Ok(db_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_settings(directory: Option<&str>, filename: Option<&str>) -> SqliteSettings {
        SqliteSettings {
            directory: directory.map(str::to_owned),
            filename: filename.map(str::to_owned),
            max_connections: 5,
            connect_timeout_secs: 10,
        }
    }

    #[test]
    fn resolve_sqlite_database_path_combines_directory_and_filename_success() {
        let settings = sample_settings(Some("/tmp/sqlbinder"), Some("app.db"));

        let path = resolve_sqlite_database_path(&settings).expect("expected sqlite database path to resolve");

        assert_eq!(path, PathBuf::from("/tmp/sqlbinder/app.db"));
    }

    #[test]
    fn resolve_sqlite_database_path_rejects_empty_directory_failure() {
        let settings = sample_settings(Some("   "), Some("app.db"));

        let error = resolve_sqlite_database_path(&settings).unwrap_err();

        assert!(matches!(
            error,
            DbConnectionError::InvalidConfiguration { field, .. } if field == "database.directory"
        ));
    }

    #[test]
    fn resolve_sqlite_database_path_rejects_missing_directory_failure() {
        let settings = sample_settings(None, Some("app.db"));

        let error = resolve_sqlite_database_path(&settings).unwrap_err();

        assert!(matches!(
            error,
            DbConnectionError::InvalidConfiguration { field, .. } if field == "database.directory"
        ));
    }

    #[test]
    fn resolve_sqlite_database_path_rejects_empty_filename_failure() {
        let settings = sample_settings(Some("/tmp/sqlbinder"), Some("   "));

        let error = resolve_sqlite_database_path(&settings).unwrap_err();

        assert!(matches!(
            error,
            DbConnectionError::InvalidConfiguration { field, .. } if field == "database.filename"
        ));
    }

    #[test]
    fn resolve_sqlite_database_path_rejects_missing_filename_failure() {
        let settings = sample_settings(Some("/tmp/sqlbinder"), None);

        let error = resolve_sqlite_database_path(&settings).unwrap_err();

        assert!(matches!(
            error,
            DbConnectionError::InvalidConfiguration { field, .. } if field == "database.filename"
        ));
    }

    #[test]
    fn resolve_sqlite_database_path_trims_valid_inputs_success() {
        let settings = sample_settings(Some("  /tmp/sqlbinder  "), Some("  app.db  "));

        let path = resolve_sqlite_database_path(&settings).expect("expected sqlite database path to resolve");

        assert_eq!(path, PathBuf::from("/tmp/sqlbinder/app.db"));
    }

    #[test]
    fn create_connection_options_rejects_empty_path_failure() {
        let settings = sample_settings(Some("/tmp/sqlbinder"), Some("   "));

        let error = create_connection_options(&settings).unwrap_err();

        assert!(matches!(
            error,
            DbConnectionError::InvalidConfiguration { field, .. } if field == "database.filename"
        ));
    }

    #[tokio::test]
    async fn get_sqlite_in_memory_pool_creates_pool_success() {
        let settings = SqliteInMemorySettings {
            max_connections: 5,
            connect_timeout_secs: 10,
        };

        let _pool = get_sqlite_in_memory_pool(&settings).expect("expected sqlite in-memory pool to be created");
    }
}
use crate::context_container::app_settings::model::DatabaseSettings;

use super::error::DbConnectionError;
use super::model::DbConnectionPool;
use crate::db::{pg, sqlite};

pub fn get_connection_pool(settings: &DatabaseSettings) -> Result<DbConnectionPool, DbConnectionError> {
    settings.validate().map_err(DbConnectionError::Settings)?;

    match settings {
        DatabaseSettings::Postgres(postgres_settings) => {
            pg::connection::get_pg_pool(postgres_settings).map(DbConnectionPool::Postgres)
        }
        DatabaseSettings::Sqlite(sqlite_settings) => {
            sqlite::connection::get_sqlite_pool(sqlite_settings).map(DbConnectionPool::Sqlite)
        }
        DatabaseSettings::SQLiteInMemory(sqlite_settings) => {
            sqlite::connection::get_sqlite_in_memory_pool(sqlite_settings).map(DbConnectionPool::Sqlite)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_container::app_settings::model::{
        PostgresSettings,
        SqliteInMemorySettings,
        SqliteSettings,
    };

    #[tokio::test]
    async fn sqlite_backend_dispatch_creates_sqlite_pool_success() {
        let settings = DatabaseSettings::Sqlite(SqliteSettings {
            directory: Some("/tmp/sqlbinder".to_owned()),
            filename: Some("app.sqlite".to_owned()),
            max_connections: 5,
            connect_timeout_secs: 10,
        });

        let pool = get_connection_pool(&settings).expect("expected sqlite pool to be created");

        assert!(matches!(pool, DbConnectionPool::Sqlite(_)));
    }

    #[tokio::test]
    async fn sqlite_in_memory_backend_dispatch_creates_sqlite_pool_success() {
        let settings = DatabaseSettings::SQLiteInMemory(SqliteInMemorySettings {
            max_connections: 5,
            connect_timeout_secs: 10,
        });

        let pool = get_connection_pool(&settings).expect("expected sqlite in-memory pool to be created");

        assert!(matches!(pool, DbConnectionPool::Sqlite(_)));
    }

    #[test]
    fn sqlite_backend_dispatch_propagates_path_validation_failure() {
        let settings = DatabaseSettings::Sqlite(SqliteSettings {
            directory: Some("/tmp/sqlbinder".to_owned()),
            filename: None,
            max_connections: 5,
            connect_timeout_secs: 10,
        });

        let result = get_connection_pool(&settings);

        assert!(matches!(
            result,
            Err(DbConnectionError::Settings(crate::context_container::app_settings::AppSettingsError::InvalidConfig(message)))
                if message == "SQLite filename is required for file-based mode"
        ));
    }

    #[test]
    fn sqlite_in_memory_backend_dispatch_propagates_validation_failure() {
        let settings = DatabaseSettings::SQLiteInMemory(SqliteInMemorySettings {
            max_connections: 0,
            connect_timeout_secs: 10,
        });

        let result = get_connection_pool(&settings);

        assert!(matches!(
            result,
            Err(DbConnectionError::Settings(crate::context_container::app_settings::AppSettingsError::InvalidConfig(message)))
                if message == "Database max_connections must be greater than 0"
        ));
    }

    #[test]
    fn sqlite_backend_dispatch_propagates_directory_validation_failure() {
        let settings = DatabaseSettings::Sqlite(SqliteSettings {
            directory: Some("   ".to_owned()),
            filename: Some("app.sqlite".to_owned()),
            max_connections: 5,
            connect_timeout_secs: 10,
        });

        let result = get_connection_pool(&settings);

        assert!(matches!(
            result,
            Err(DbConnectionError::Settings(crate::context_container::app_settings::AppSettingsError::InvalidConfig(message)))
                if message == "SQLite directory is required for file-based mode"
        ));
    }

    #[test]
    fn postgres_backend_dispatch_requires_password_source_failure() {
        let settings = DatabaseSettings::Postgres(PostgresSettings {
            host: "localhost".to_owned(),
            port: 5432,
            user: "postgres".to_owned(),
            name: "sqlbinder".to_owned(),
            password_file: None,
            max_connections: 5,
            connect_timeout_secs: 10,
            ssl_mode: "prefer".to_owned(),
        });

        assert!(matches!(
            get_connection_pool(&settings),
            Err(DbConnectionError::Settings(crate::context_container::app_settings::AppSettingsError::InvalidConfig(message)))
                if message == "Database password_file is required but not provided"
        ));
    }
}
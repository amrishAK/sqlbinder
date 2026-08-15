use std::sync::Arc;

use super::ContextContainer;
use super::ContextContainerError;
use super::app_settings::{AppSettings, DatabaseSettings};
use crate::db::DbConnectionPool;

impl ContextContainer {

    /// Create a new application context by loading settings.
    /// 
    /// Returns an error if settings loading fails.
    pub fn new() -> Result<Self, ContextContainerError> {
        let settings = AppSettings::load_settings()
            .map_err(ContextContainerError::Settings)?;
        Ok(ContextContainer {
            app_settings: Arc::new(settings),
            connection_pool: Arc::new(None),
        })
    }

    /// Store the shared database connection pool in the context.
    pub fn set_connection_pool(&mut self, connection_pool: DbConnectionPool) {
        self.connection_pool = Arc::new(Some(connection_pool));
    }

    /// Return the shared database connection pool if one has been initialized.
    pub fn get_connection_pool(&self) -> Option<DbConnectionPool> {
        self.connection_pool.as_ref().clone()
    }

    /// Return a cloned database settings snapshot wrapped in an Arc.
    pub fn get_database_settings(&self) -> Arc<DatabaseSettings> {
        Arc::new(self.get_app_settings().database.clone())
    }

    /// Read the configured environment name from settings.
    pub fn get_environment_name(&self) -> String {
        self.app_settings.environment.clone()
    }

    fn get_app_settings(&self) -> Arc<AppSettings> {
        Arc::clone(&self.app_settings)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::super::app_settings::{DatabaseSettings, PostgresSettings};
    use super::*;
    use crate::context_container::shared_cwd_test_lock;

    fn test_app_settings(environment: &str) -> AppSettings {
        AppSettings {
            environment: environment.to_owned(),
            database: DatabaseSettings::Postgres(PostgresSettings {
                host: "localhost".to_owned(),
                port: 5432,
                user: "postgres".to_owned(),
                name: "sqlbinder".to_owned(),
                password_file: Some("test-utils/unit-test-resources/secrets/db_password_with_newline.txt".to_owned()),
                max_connections: 5,
                connect_timeout_secs: 10,
                ssl_mode: "disable".to_owned(),
            }),
        }
    }

    #[test]
    fn get_environment_name_returns_loaded_environment_success() {
        let container = ContextContainer {
            app_settings: Arc::new(test_app_settings("development")),
            connection_pool: Arc::new(None),
        };

        let environment_name = container.get_environment_name();

        assert_eq!(environment_name, "development");
    }

    #[test]
    fn get_database_settings_returns_snapshot_success() {
        let container = ContextContainer {
            app_settings: Arc::new(test_app_settings("production")),
            connection_pool: Arc::new(None),
        };

        let database_settings = container.get_database_settings();
        let database = database_settings
            .postgres()
            .expect("expected postgres settings");

        assert_eq!(database.host, "localhost");
        assert_eq!(database.port, 5432);
        assert_eq!(database.user, "postgres");
        assert_eq!(database.password_file.as_deref(), Some("test-utils/unit-test-resources/secrets/db_password_with_newline.txt"));
    }

    #[test]
    fn constructor_loads_default_settings_file_success() {
        let _lock = shared_cwd_test_lock();
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let settings_path = crate_root.join("settings.toml");
        let settings_backup = fs::read_to_string(&settings_path).ok();

        let settings_contents = r#"
        environment = "development"
        database_type = "postgres"

        [database]
        host = "localhost"
        port = 5432
        user = "appuser"
        name = "appdb"
        password_file = "test-utils/unit-test-resources/secrets/db_password_with_newline.txt"
        max_connections = 10
        connect_timeout_secs = 5
        ssl_mode = "disable"
        "#;

        fs::write(&settings_path, settings_contents).expect("settings file should be written");

        let container = ContextContainer::new().expect("default settings should load from settings.toml");

        if let Some(content) = settings_backup {
            fs::write(&settings_path, content).expect("settings backup should be restored");
        } else {
            let _ = fs::remove_file(&settings_path);
        }

        assert_eq!(container.get_environment_name(), "development");
        let database_settings = container.get_database_settings();
        let database = database_settings
            .postgres()
            .expect("expected postgres settings");
        assert_eq!(database.host, "localhost");
        assert_eq!(database.name, "appdb");
    }

    #[test]
    fn get_connection_pool_returns_none_before_initialization_success() {
        let container = ContextContainer {
            app_settings: Arc::new(test_app_settings("development")),
            connection_pool: Arc::new(None),
        };

        assert!(container.get_connection_pool().is_none());
    }

    #[tokio::test]
    async fn set_connection_pool_stores_pool_success() {
        let mut container = ContextContainer {
            app_settings: Arc::new(test_app_settings("development")),
            connection_pool: Arc::new(None),
        };
        let pool = DbConnectionPool::Sqlite(
            crate::db::SqlitePool::new(sqlx::sqlite::SqlitePoolOptions::new().connect_lazy_with(
                sqlx::sqlite::SqliteConnectOptions::new().in_memory(true),
            )),
        );

        container.set_connection_pool(pool.clone());

        assert!(matches!(container.get_connection_pool(), Some(DbConnectionPool::Sqlite(_))));
    }

    #[tokio::test]
    async fn cloned_container_shares_connection_pool_success() {
        let mut container = ContextContainer {
            app_settings: Arc::new(test_app_settings("development")),
            connection_pool: Arc::new(None),
        };
        let pool = DbConnectionPool::Sqlite(
            crate::db::SqlitePool::new(sqlx::sqlite::SqlitePoolOptions::new().connect_lazy_with(
                sqlx::sqlite::SqliteConnectOptions::new().in_memory(true),
            )),
        );
        container.set_connection_pool(pool);

        let cloned = container.clone();

        assert!(matches!(cloned.get_connection_pool(), Some(DbConnectionPool::Sqlite(_))));
    }
}

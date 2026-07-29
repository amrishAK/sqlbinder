use std::sync::Arc;

use super::ContextContainer;
use super::ContextContainerError;
use super::app_settings::AppSettings;

impl ContextContainer {

    /// Create a new application context by loading settings.
    /// 
    /// Returns an error if settings loading fails.
    pub fn new() -> Result<Self, ContextContainerError> {
        let settings = AppSettings::load_settings()
            .map_err(ContextContainerError::Settings)?;
        Ok(ContextContainer {
            app_settings: Arc::new(settings),
        })
    }

    /// Return a shared handle to application settings.
    pub fn get_app_settings(&self) -> Arc<AppSettings> {
        self.app_settings.clone()
    }

    /// Read the configured environment name from settings.
    pub fn get_environment_name(&self) -> String {
        self.app_settings.environment.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::app_settings::DatabaseSettings;
    use super::*;

    fn test_app_settings(environment: &str) -> AppSettings {
        AppSettings {
            environment: environment.to_owned(),
            database: DatabaseSettings {
                host: "localhost".to_owned(),
                port: 5432,
                user: "postgres".to_owned(),
                name: "sqlbinder".to_owned(),
                password_file: None,
                max_connections: 5,
                connect_timeout_secs: 10,
                ssl_mode: "disable".to_owned(),
            },
        }
    }

    #[test]
    fn get_environment_name_returns_loaded_environment_success() {
        let app_settings = test_app_settings("development");
        let container = ContextContainer {
            app_settings: Arc::new(app_settings),
        };

        let environment_name = container.get_environment_name();

        assert_eq!(environment_name, "development");
    }

    #[test]
    fn get_app_settings_returns_cloned_arc_reference_success() {
        let app_settings = test_app_settings("production");
        let container = ContextContainer {
            app_settings: Arc::new(app_settings),
        };

        let retrieved = container.get_app_settings();

        assert_eq!(retrieved.environment, "production");
        assert_eq!(retrieved.database.host, "localhost");
        assert_eq!(retrieved.database.port, 5432);
    }

    #[test]
    fn get_app_settings_returns_shared_reference_success() {
        let app_settings = test_app_settings("staging");
        let container = ContextContainer {
            app_settings: Arc::new(app_settings),
        };

        let retrieved1 = container.get_app_settings();
        let retrieved2 = container.get_app_settings();

        // Both references point to the same underlying data
        assert_eq!(retrieved1.environment, retrieved2.environment);
        assert!(Arc::ptr_eq(&retrieved1, &retrieved2));
    }

    #[test]
    fn constructor_pattern_creates_initialized_container_success() {
        // Simulate what the constructor does: load settings and wrap in Arc
        let app_settings = test_app_settings("test");
        let container = ContextContainer {
            app_settings: Arc::new(app_settings),
        };

        // Verify the container is fully initialized and ready to use
        let settings = container.get_app_settings();
        assert_eq!(settings.environment, "test");
        assert_eq!(settings.database.port, 5432);
    }

    #[test]
    fn constructor_ensures_settings_always_present_success() {
        // The new constructor pattern guarantees settings are always present
        let app_settings = test_app_settings("production");
        let container = ContextContainer {
            app_settings: Arc::new(app_settings),
        };

        // Both getters should work without any Result unwrapping
        let environment = container.get_environment_name();
        let settings = container.get_app_settings();

        // Settings are always accessible, making invalid states impossible
        assert_eq!(environment, settings.environment);
        assert!(!settings.database.host.is_empty());
    }

    #[test]
    fn container_with_valid_test_data_success() {
        // Create settings matching the base_valid.toml test resource structure
        let app_settings = AppSettings {
            environment: "development".to_owned(),
            database: DatabaseSettings {
                host: "localhost".to_owned(),
                port: 5432,
                user: "appuser".to_owned(),
                name: "appdb".to_owned(),
                password_file: Some("test-utils/unit-test-resources/secrets/db_password_with_newline.txt".to_owned()),
                max_connections: 10,
                connect_timeout_secs: 5,
                ssl_mode: "disable".to_owned(),
            },
        };
        let container = ContextContainer {
            app_settings: Arc::new(app_settings),
        };

        let retrieved_env = container.get_environment_name();
        let retrieved_settings = container.get_app_settings();

        assert_eq!(retrieved_env, "development");
        assert_eq!(retrieved_settings.database.host, "localhost");
        assert_eq!(retrieved_settings.database.port, 5432);
        assert_eq!(retrieved_settings.database.user, "appuser");
        assert_eq!(retrieved_settings.database.max_connections, 10);
    }

    #[test]
    fn container_preserves_password_file_path_success() {
        let app_settings = AppSettings {
            environment: "production".to_owned(),
            database: DatabaseSettings {
                host: "db.example.com".to_owned(),
                port: 5432,
                user: "produser".to_owned(),
                name: "proddb".to_owned(),
                password_file: Some("test-utils/unit-test-resources/secrets/db_password_with_newline.txt".to_owned()),
                max_connections: 20,
                connect_timeout_secs: 10,
                ssl_mode: "require".to_owned(),
            },
        };
        let container = ContextContainer {
            app_settings: Arc::new(app_settings),
        };

        let settings = container.get_app_settings();

        // Verify password file path is accessible through container
        assert_eq!(
            settings.database.password_file.as_deref(),
            Some("test-utils/unit-test-resources/secrets/db_password_with_newline.txt")
        );
    }
}

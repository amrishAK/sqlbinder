use std::sync::Arc;

use crate::utils::AppSettings;
use crate::utils::AppSettingsError;

use super::{ContextContainer, ContextContainerError};

impl ContextContainer {

    /// Load application settings once and store them in shared context.
    ///
    /// Returns an error when settings were already initialized or loading fails.
    pub fn load_app_settings(&mut self) -> Result<(), ContextContainerError> {
        
        if self.app_settings.is_some() {
            return Err(ContextContainerError::SettingsAlreadyInitialized);
        }

        let settings = AppSettings::load_settings().map_err(ContextContainerError::Settings)?;
        self.app_settings = Some(Arc::new(settings));
        Ok(())
    }

    /// Return a shared handle to initialized application settings.
    pub fn get_app_settings(&self) -> Result<Arc<AppSettings>, ContextContainerError> {
        self.fetch_app_settings()
    }

    /// Read the configured environment name from initialized settings.
    pub fn get_environment_name(&self) -> Result<String, ContextContainerError> {
        let settings = self.fetch_app_settings()?;
        Ok(settings.environment.clone())
    }

    fn fetch_app_settings(&self) -> Result<Arc<AppSettings>, ContextContainerError> {
        self.app_settings
            .as_ref()
            .cloned()
            .ok_or(ContextContainerError::Settings(
                AppSettingsError::SettingsNotInitialized,
            ))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::utils::DatabaseSettings;

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
    fn get_environment_name_success() {
        let container = ContextContainer {
            app_settings: Some(Arc::new(test_app_settings("development"))),
        };

        let environment_name = container.get_environment_name().unwrap();

        assert_eq!(environment_name, "development");
    }

    #[test]
    fn get_app_settings_without_initialization_failure() {
        let container = ContextContainer { app_settings: None };

        let err = container.get_app_settings().unwrap_err();

        assert!(matches!(
            err,
            ContextContainerError::Settings(AppSettingsError::SettingsNotInitialized)
        ));
    }
}

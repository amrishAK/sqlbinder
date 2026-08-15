/// Application settings loading and data models.
pub mod app_settings;
/// Error types for context container initialization and access.
pub mod error;
/// Context container data model shared across application layers.
pub mod model;
/// APIs for loading and reading values from the context container.
pub mod context_container_handler;

#[cfg(test)]
pub(crate) fn shared_cwd_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Public error type for context container operations.
pub use error::ContextContainerError;
/// Public context container type.
pub use model::ContextContainer;
/// Public settings types and errors.
pub use app_settings::{AppSettings, AppSettingsError, DatabaseSettings, DEFAULT_SETTINGS_FILE};
/// Error types returned by settings and secret loading utilities.
pub mod error;
/// Settings loading and singleton access helpers.
pub mod settings_handler;
/// Settings data structures and defaults.
pub mod model;
/// Secret file loading helpers.
pub mod secret_handler;

/// Public settings error type.
pub use error::AppSettingsError;
/// Public settings models and default settings file name.
pub use model::{AppSettings, DatabaseSettings, DEFAULT_SETTINGS_FILE};

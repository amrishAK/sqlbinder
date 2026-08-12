/// Error types returned by settings and secret loading utilities.
pub mod error;
/// Settings loading and singleton access helpers.
pub mod settings_handler;
/// Settings data structures and defaults.
pub mod model;
/// Implementations for settings models.
mod database_settings_handler;
/// Raw TOML parsing and conversion helpers.
pub mod raw_settings_handler;
/// Shared settings validation helpers.
mod settings_validator;
/// Secret file loading helpers.
pub mod secret_handler;

/// Public settings error type.
pub use error::AppSettingsError;
/// Public settings models and default settings file name.
pub use model::{
	AppSettings,
	DatabaseSettings,
	DbType,
	PostgresSettings,
	SqliteInMemorySettings,
	SqliteSettings,
	DEFAULT_SETTINGS_FILE,
};

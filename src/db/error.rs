use thiserror::Error;

use crate::context_container::app_settings::AppSettingsError;

#[derive(Debug, Error)]
pub enum DbConnectionError {
	#[error("invalid database configuration for '{field}': {message}")]
	InvalidConfiguration {
		field: &'static str,
		message: String,
	},
	#[error("unsupported postgres ssl_mode '{0}'")]
	UnsupportedPostgresSslMode(String),
	#[error("missing password source for backend '{backend}' (password_file)")]
	MissingPasswordSource {
		backend: &'static str,
	},
	#[error("sqlite database name/path cannot be empty")]
	EmptySqliteDatabaseName,
	#[error("database settings error: {0}")]
	Settings(#[from] AppSettingsError),
	#[error("database initialization failed: {0}")]
	Initialization(#[source] sqlx::Error),
}

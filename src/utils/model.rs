use serde::Deserialize;

/// Default settings file loaded from the current working directory.
pub const DEFAULT_SETTINGS_FILE: &str = "settings.toml";
/// Environment variable that can identify the active runtime environment.
pub const ENVIRONMENT_ENV_VAR: &str = "ENVIRONMENT";

fn default_environment() -> String {
	"development".to_owned()
}

#[derive(Debug, Clone, Deserialize)]
/// Root application settings loaded from TOML files.
pub struct AppSettings {
	#[serde(default = "default_environment")]
	pub environment: String,
	pub database: DatabaseSettings,
}

#[derive(Debug, Clone, Deserialize)]
/// Database connection settings resolved from application configuration.
pub struct DatabaseSettings {
	pub host: String,
	pub port: u16,
	pub user: String,
	pub name: String,
	pub password_file: Option<String>,
	pub max_connections: u32,
	pub connect_timeout_secs: u64,
	pub ssl_mode: String,
}

/// Default settings file loaded from the current working directory.
pub const DEFAULT_SETTINGS_FILE: &str = "settings.toml";
/// Environment variable that can identify the active runtime environment.
pub const ENVIRONMENT_ENV_VAR: &str = "ENVIRONMENT";

/// Database backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbType {
	PostgreSQL,
	SQLite,
	SQLiteInMemory,
}

#[derive(Debug, Clone)]
/// Root application settings loaded from TOML files.
pub struct AppSettings {
	pub environment: String,
	pub database: DatabaseSettings,
}

#[derive(Debug, Clone)]
/// Database settings resolved from application configuration.
pub enum DatabaseSettings {
	Postgres(PostgresSettings),
	Sqlite(SqliteSettings),
	SQLiteInMemory(SqliteInMemorySettings),
}

impl DatabaseSettings {
	pub fn postgres(&self) -> Option<&PostgresSettings> {
		match self {
			DatabaseSettings::Postgres(settings) => Some(settings),
			_ => None,
		}
	}

	pub fn sqlite(&self) -> Option<&SqliteSettings> {
		match self {
			DatabaseSettings::Sqlite(settings) => Some(settings),
			_ => None,
		}
	}

	pub fn sqlite_in_memory(&self) -> Option<&SqliteInMemorySettings> {
		match self {
			DatabaseSettings::SQLiteInMemory(settings) => Some(settings),
			_ => None,
		}
	}
}

#[derive(Debug, Clone)]
/// PostgreSQL-specific settings resolved from application configuration.
pub struct PostgresSettings {
	pub host: String,
	pub port: u16,
	pub user: String,
	pub name: String,
	pub password_file: Option<String>,
	pub max_connections: u32,
	pub connect_timeout_secs: u64,
	pub ssl_mode: String,
}

#[derive(Debug, Clone)]
/// SQLite-specific settings resolved from application configuration.
pub struct SqliteSettings {
	pub directory: Option<String>,
	pub filename: Option<String>,
	pub max_connections: u32,
	pub connect_timeout_secs: u64,
}

#[derive(Debug, Clone)]
/// SQLite in-memory-specific settings resolved from application configuration.
pub struct SqliteInMemorySettings {
	pub max_connections: u32,
	pub connect_timeout_secs: u64,
}


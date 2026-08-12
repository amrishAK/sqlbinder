use super::error::AppSettingsError;
use super::model::{
	DatabaseSettings,
	DbType,
	PostgresSettings,
	SqliteInMemorySettings,
	SqliteSettings,
};
use super::secret_handler;
use super::settings_validator::{optional_string, required_string, required_u16, required_u32, required_u64};

impl DbType {
	/// Convert to a simple string representation.
	pub fn as_str(&self) -> &'static str {
		match self {
			DbType::PostgreSQL => "postgres",
			DbType::SQLite => "sqlite",
			DbType::SQLiteInMemory => "in-memory",
		}
	}

	pub(super) fn from_str(value: &str) -> Result<Self, AppSettingsError> {
		match value.to_lowercase().as_str() {
			"postgres"  => Ok(DbType::PostgreSQL),
			"sqlite"  => Ok(DbType::SQLite),
			"in-memory" | "memory" | "sqlite-in-memory" => Ok(DbType::SQLiteInMemory),
			_ => Err(AppSettingsError::InvalidConfig(format!(
				"invalid database backend: {value}"
			))),
		}
	}
}



impl DatabaseSettings {
	
	pub(super) fn from_toml_table(table: &toml::Table, backend: DbType) -> Result<Self, AppSettingsError> {
		match backend {
			DbType::PostgreSQL => Self::validate_and_get_postgres_settings(table)
				.map(DatabaseSettings::Postgres),
			DbType::SQLite => Self::validate_and_get_sqlite_settings(table)
				.map(DatabaseSettings::Sqlite),
			DbType::SQLiteInMemory => Self::get_and_validate_sqlite_in_memory_settings(table)
				.map(DatabaseSettings::SQLiteInMemory),
		}
	}


	fn validate_and_get_postgres_settings(table: &toml::Table) -> Result<PostgresSettings, AppSettingsError> {
		let settings = PostgresSettings {
			host: required_string(table, "host", "host is required and must be a string")?,
			port: required_u16(table, "port", "port is required and must be a valid u16 integer")?,
			user: required_string(table, "user", "user is required and must be a string")?,
			name: required_string(table, "name", "name is required and must be a string")?,
			password_file: Some(required_string(table, "password_file", "password_file is required and must be a string")?),
			max_connections: required_u32(table, "max_connections", "max_connections is required and must be a valid u32 integer")?,
			connect_timeout_secs: required_u64(table, "connect_timeout_secs", "connect_timeout_secs is required and must be a valid u64 integer")?,
			ssl_mode: required_string(table, "ssl_mode", "ssl_mode is required and must be a string")?,
		};

		Self::validate_postgres_settings(&settings)?;
		Ok(settings)
	}

	fn validate_and_get_sqlite_settings(table: &toml::Table) -> Result<SqliteSettings, AppSettingsError> {
		let settings = SqliteSettings {
			directory: Some(required_string(table, "directory", "directory is required and must be a string")?),
			filename: optional_string(table, "filename"),
			max_connections: required_u32(table, "max_connections", "max_connections is required and must be a valid u32 integer")?,
			connect_timeout_secs: required_u64(table, "connect_timeout_secs", "connect_timeout_secs is required and must be a valid u64 integer")?,
		};

		Self::validate_sqlite_settings(&settings)?;
		Ok(settings)
	}

	fn get_and_validate_sqlite_in_memory_settings(table: &toml::Table) -> Result<SqliteInMemorySettings, AppSettingsError> {
		let settings = SqliteInMemorySettings {
			max_connections: required_u32(table, "max_connections", "max_connections is required and must be a valid u32 integer")?,
			connect_timeout_secs: required_u64(table, "connect_timeout_secs", "connect_timeout_secs is required and must be a valid u64 integer")?,
		};

		Self::validate_sqlite_in_memory_settings(&settings)?;
		Ok(settings)
	}

	fn validate_sqlite_settings(settings: &SqliteSettings) -> Result<(), AppSettingsError> {
		let directory = settings.directory.as_deref().map(str::trim).unwrap_or("");
		let filename = settings.filename.as_deref().map(str::trim).unwrap_or("");

		if directory.is_empty() {
			return Err(AppSettingsError::InvalidConfig(
				"SQLite directory is required for file-based mode".to_string(),
			));
		}

		if filename.is_empty() {
			return Err(AppSettingsError::InvalidConfig(
				"SQLite filename is required for file-based mode".to_string(),
			));
		}

		if filename.contains('/') || filename.contains('\\') {
			return Err(AppSettingsError::InvalidConfig(
				"SQLite filename cannot include path separators".to_string(),
			));
		}

		if filename == "." || filename == ".." {
			return Err(AppSettingsError::InvalidConfig(
				"SQLite filename cannot be '.' or '..'".to_string(),
			));
		}

		if settings.max_connections == 0 {
			return Err(AppSettingsError::InvalidConfig(
				"Database max_connections must be greater than 0".to_string(),
			));
		}

		if settings.connect_timeout_secs == 0 {
			return Err(AppSettingsError::InvalidConfig(
				"Database connect_timeout_secs must be greater than 0".to_string(),
			));
		}

		Ok(())
	}

	fn validate_sqlite_in_memory_settings(settings: &SqliteInMemorySettings) -> Result<(), AppSettingsError> {
		if settings.max_connections == 0 {
			return Err(AppSettingsError::InvalidConfig(
				"Database max_connections must be greater than 0".to_string(),
			));
		}

		if settings.connect_timeout_secs == 0 {
			return Err(AppSettingsError::InvalidConfig(
				"Database connect_timeout_secs must be greater than 0".to_string(),
			));
		}

		Ok(())
	}


	fn validate_postgres_settings(settings: &PostgresSettings) -> Result<(), AppSettingsError> {
		if settings.host.is_empty() {
			return Err(AppSettingsError::InvalidConfig(
				"Database host is required but not provided".to_string(),
			));
		}

		if settings.user.is_empty() {
			return Err(AppSettingsError::InvalidConfig(
				"Database user is required but not provided".to_string(),
			));
		}

		if settings.name.is_empty() {
			return Err(AppSettingsError::InvalidConfig(
				"Database name is required but not provided".to_string(),
			));
		}

		// Fail-fast: verify the configured password file can be read.
		secret_handler::validate_required_secret_file(settings.password_file.as_deref())?;

		if settings.max_connections == 0 {
			return Err(AppSettingsError::InvalidConfig(
				"Database max_connections must be greater than 0".to_string(),
			));
		}

		if settings.connect_timeout_secs == 0 {
			return Err(AppSettingsError::InvalidConfig(
				"Database connect_timeout_secs must be greater than 0".to_string(),
			));
		}

		if settings.ssl_mode.is_empty() {
			return Err(AppSettingsError::InvalidConfig(
				"Database ssl_mode is required but not provided".to_string(),
			));
		}

		Ok(())
	}

	/// Validate database settings, ensuring all required fields are properly configured.
	pub fn validate(&self) -> Result<(), AppSettingsError> {
		match self {
			DatabaseSettings::Postgres(settings) => Self::validate_postgres_settings(settings),
			DatabaseSettings::Sqlite(settings) => Self::validate_sqlite_settings(settings),
			DatabaseSettings::SQLiteInMemory(settings) => Self::validate_sqlite_in_memory_settings(settings),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sqlite_file_settings_validate_success() {
		let settings = DatabaseSettings::Sqlite(SqliteSettings {
			directory: Some("data".to_owned()),
			filename: Some("app.sqlite".to_owned()),
			max_connections: 4,
			connect_timeout_secs: 10,
		});

		assert!(settings.validate().is_ok());
	}

	#[test]
	fn sqlite_memory_settings_validate_success() {
		let settings = DatabaseSettings::SQLiteInMemory(SqliteInMemorySettings {
			max_connections: 4,
			connect_timeout_secs: 10,
		});

		assert!(settings.validate().is_ok());
	}

	#[test]
	fn sqlite_memory_settings_reject_zero_connections_failure() {
		let settings = DatabaseSettings::SQLiteInMemory(SqliteInMemorySettings {
			max_connections: 4,
			connect_timeout_secs: 10,
		});

		assert!(settings.validate().is_ok());

		let settings = DatabaseSettings::SQLiteInMemory(SqliteInMemorySettings {
			max_connections: 0,
			connect_timeout_secs: 10,
		});

		assert!(matches!(
			settings.validate(),
			Err(AppSettingsError::InvalidConfig(message)) if message == "Database max_connections must be greater than 0"
		));
	}

	#[test]
	fn sqlite_missing_filename_validate_failure() {
		let settings = DatabaseSettings::Sqlite(SqliteSettings {
			directory: Some("data".to_owned()),
			filename: None,
			max_connections: 4,
			connect_timeout_secs: 10,
		});

		assert!(matches!(
			settings.validate(),
			Err(AppSettingsError::InvalidConfig(message)) if message == "SQLite filename is required for file-based mode"
		));
	}

	#[test]
	fn sqlite_filename_with_separator_validate_failure() {
		let settings = DatabaseSettings::Sqlite(SqliteSettings {
			directory: Some("data".to_owned()),
			filename: Some("nested/app.sqlite".to_owned()),
			max_connections: 4,
			connect_timeout_secs: 10,
		});

		assert!(matches!(
			settings.validate(),
			Err(AppSettingsError::InvalidConfig(message)) if message == "SQLite filename cannot include path separators"
		));
	}

	#[test]
	fn sqlite_filename_parent_reference_validate_failure() {
		let settings = DatabaseSettings::Sqlite(SqliteSettings {
			directory: Some("data".to_owned()),
			filename: Some("..".to_owned()),
			max_connections: 4,
			connect_timeout_secs: 10,
		});

		assert!(matches!(
			settings.validate(),
			Err(AppSettingsError::InvalidConfig(message)) if message == "SQLite filename cannot be '.' or '..'"
		));
	}

	#[test]
	fn sqlite_file_settings_reject_empty_directory_failure() {
		let settings = DatabaseSettings::Sqlite(SqliteSettings {
			directory: Some("   ".to_owned()),
			filename: Some("app.sqlite".to_owned()),
			max_connections: 4,
			connect_timeout_secs: 10,
		});

		assert!(matches!(
			settings.validate(),
			Err(AppSettingsError::InvalidConfig(message)) if message == "SQLite directory is required for file-based mode"
		));
	}
}

#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::Path;

use secrecy::SecretString;

use super::error::AppSettingsError;
use super::model::{
	AppSettings,
	DatabaseSettings,
};
use super::model::DbType;

use super::raw_settings_handler::from_default_file;

use super::secret_handler::load_secret_from_file;

impl AppSettings {

	pub fn load_settings() -> Result<AppSettings, AppSettingsError> {
		let raw_value = from_default_file()?;
		let settings = AppSettings::from_toml_value(raw_value)?;
		Ok(settings)
	}

	pub fn from_toml_value(raw: toml::Value) -> Result<Self, AppSettingsError> {
		
		let environment = Self::validate_and_get_environment(&raw)?;
		let database_type = Self::validate_and_get_database_type(&raw)?;

		let database_table = raw
			.get("database")
			.and_then(toml::Value::as_table)
			.ok_or_else(|| AppSettingsError::InvalidConfig("database table is required".to_owned()))?;

		let database = DatabaseSettings::from_toml_table(database_table, database_type)?;
		let settings = Self { environment, database };
		Ok(settings)
	}

	fn validate_and_get_environment(raw: &toml::Value) -> Result<String, AppSettingsError> {
		let environment = raw
			.get("environment")
			.and_then(toml::Value::as_str)
			.map(str::to_owned)
			.unwrap_or_else(|| "local".to_owned());

		if environment.is_empty() {
			return Err(AppSettingsError::InvalidConfig(
				"Application environment is required but not provided".to_string(),
			));
		}

		let valid_environments = ["local", "development", "qa", "production"];
		if !valid_environments.contains(&environment.as_str()) {
			return Err(AppSettingsError::InvalidConfig(
				format!(
					"Invalid environment '{}': must be one of {:?}",
					environment, valid_environments
				),
			));
		}

		Ok(environment)
	}

	fn validate_and_get_database_type(raw: &toml::Value) -> Result<DbType, AppSettingsError> {
		let database_type = raw
			.get("database_type")
			.and_then(toml::Value::as_str)
			.ok_or_else(|| AppSettingsError::InvalidConfig("database_type is required".to_owned()))?;

		DbType::from_str(database_type)
	}

	/// Load and deserialize settings from the default settings file and any environment overlay.
	pub fn from_default_file() -> Result<Self, AppSettingsError> {
		let raw_value = from_default_file()?;
		Self::from_toml_value(raw_value)
	}

	/// Load and deserialize settings from a specific TOML file.
	#[cfg(test)]
	fn from_file(path: impl AsRef<Path>) -> Result<Self, AppSettingsError> {
		let raw = fs::read_to_string(path)?;
		let value: toml::Value = toml::from_str(&raw)?;
		Self::from_toml_value(value)
	}

	/// Load the database password from the configured password file.
	pub fn get_database_password(&self) -> Result<SecretString, AppSettingsError> {
		let path = match &self.database {
			DatabaseSettings::Postgres(settings) => settings
				.password_file
				.as_deref()
				.ok_or(AppSettingsError::MissingPasswordSource)?,
			DatabaseSettings::Sqlite(_) => return Err(AppSettingsError::MissingPasswordSource),
			DatabaseSettings::SQLiteInMemory(_) => return Err(AppSettingsError::MissingPasswordSource),
		};

		load_secret_from_file(path)
	}
}

#[cfg(test)]
mod tests {
	use std::fs;
	use std::path::PathBuf;
	use std::sync::{Mutex, MutexGuard, OnceLock};
	use std::time::{SystemTime, UNIX_EPOCH};

	use secrecy::ExposeSecret;

	use super::*;
	use crate::context_container::DatabaseSettings;
	use crate::context_container::app_settings::PostgresSettings;
	use crate::context_container::app_settings::model::DEFAULT_SETTINGS_FILE;

	fn fixture_path(relative: &str) -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("test-utils")
			.join("unit-test-resources")
			.join(relative)
	}

	fn settings_file_path(relative: &str) -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join(relative)
	}

	fn cwd_test_lock() -> MutexGuard<'static, ()> {
		static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
		LOCK.get_or_init(|| Mutex::new(()))
			.lock()
			.unwrap_or_else(|poisoned| poisoned.into_inner())
	}

	fn unique_temp_dir(prefix: &str) -> PathBuf {
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("system time should be after unix epoch")
			.as_nanos();
		std::env::temp_dir().join(format!("sqlbinder-{prefix}-{now}"))
	}

	fn with_restored_cwd<T>(f: impl FnOnce() -> T) -> T {
		let original_cwd = std::env::current_dir().expect("current directory should be readable");
		let result = f();
		std::env::set_current_dir(&original_cwd).expect("should restore original working directory");
		result
	}

	#[test]
	fn raw_toml_value_is_converted_to_structured_settings_success() {
		let raw = toml::from_str::<toml::Value>(
			r#"
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
		"#,
		)
		.expect("raw toml should parse");

		let settings = AppSettings::from_toml_value(raw).expect("raw toml should convert");
		let database = settings.database.postgres().expect("expected postgres settings");

		assert_eq!(settings.environment, "development");
		assert_eq!(database.host, "localhost");
		assert_eq!(database.port, 5432);
		assert_eq!(database.user, "appuser");
	}

	#[test]
	fn valid_settings_file_parsing_success() {
		let settings_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("test-utils")
			.join("unit-test-resources")
			.join("settings")
			.join("base_valid.toml");

		let settings = with_restored_cwd(|| AppSettings::from_file(&settings_file))
			.expect("expected valid settings fixture");
		let database = settings
			.database
			.postgres()
			.expect("expected postgres settings");

		assert_eq!(settings.environment, "development");
		assert_eq!(database.host, "localhost");
		assert_eq!(database.port, 5432);
		assert_eq!(database.user, "appuser");
		assert_eq!(database.name, "appdb");
		assert_eq!(database.max_connections, 10);
		assert_eq!(database.connect_timeout_secs, 5);
		assert_eq!(database.ssl_mode, "disable");
	}

	#[test]
	fn missing_environment_defaults_to_local_success() {
		let settings_file = settings_file_path("test-utils/unit-test-resources/settings/base_without_environment.toml");

		let settings = with_restored_cwd(|| AppSettings::from_file(&settings_file))
			.expect("expected valid settings fixture");

		assert_eq!(settings.environment, "local");
	}

	#[test]
	fn sqlite_in_memory_database_type_keywords_parse_success() {
		let raw_in_memory = toml::from_str::<toml::Value>(
			r#"
		environment = "local"
		database_type = "in-memory"

		[database]
		max_connections = 10
		connect_timeout_secs = 5
		"#,
		)
		.expect("raw toml should parse");

		let settings = AppSettings::from_toml_value(raw_in_memory)
			.expect("in-memory should parse as sqlite in-memory backend");
		assert!(matches!(settings.database, DatabaseSettings::SQLiteInMemory(_)));

		let raw_legacy_memory = toml::from_str::<toml::Value>(
			r#"
		environment = "local"
		database_type = "memory"

		[database]
		max_connections = 10
		connect_timeout_secs = 5
		"#,
		)
		.expect("raw toml should parse");

		let legacy_settings = AppSettings::from_toml_value(raw_legacy_memory)
			.expect("legacy memory alias should remain supported");
		assert!(matches!(legacy_settings.database, DatabaseSettings::SQLiteInMemory(_)));
	}

	#[test]
	fn invalid_toml_settings_file_failure() {
		let settings_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("test-utils")
			.join("unit-test-resources")
			.join("settings")
			.join("invalid.toml");

		let result = with_restored_cwd(|| AppSettings::from_file(&settings_file));

		assert!(matches!(result, Err(AppSettingsError::InvalidConfig(message)) if message.contains("database_type")));
	}

	#[test]
	fn default_file_and_environment_override_merge_success() {
		let _guard = cwd_test_lock();
		let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		let base_path = crate_root.join(DEFAULT_SETTINGS_FILE);
		let overlay_path = crate_root.join("settings.development.toml");
		let base_backup = fs::read_to_string(&base_path).ok();
		let overlay_backup = fs::read_to_string(&overlay_path).ok();

		let base_fixture = settings_file_path("test-utils/unit-test-resources/settings/default_base.toml");
		let env_fixture = settings_file_path("test-utils/unit-test-resources/settings/default_development_override.toml");
		let base_content = fs::read_to_string(&base_fixture).expect("base fixture should be readable");
		let env_content = fs::read_to_string(&env_fixture).expect("env fixture should be readable");
		let password_file = settings_file_path("test-utils/unit-test-resources/secrets/db_password_with_newline.txt")
			.to_string_lossy()
			.replace('\\', "\\\\");
		let base_content = base_content.replace(
			"test-utils/unit-test-resources/secrets/db_password_with_newline.txt",
			&password_file,
		);
		let env_content = env_content.replace(
			"test-utils/unit-test-resources/secrets/db_password_with_newline.txt",
			&password_file,
		);

		fs::write(&base_path, &base_content)
			.expect("base settings file should be written");
		fs::write(&overlay_path, &env_content)
			.expect("env overlay settings file should be written");

		let result = AppSettings::from_default_file();

		if let Some(content) = base_backup {
			fs::write(&base_path, content).expect("base settings backup should be restored");
		} else {
			let _ = fs::remove_file(&base_path);
		}

		if let Some(content) = overlay_backup {
			fs::write(&overlay_path, content).expect("overlay settings backup should be restored");
		} else {
			let _ = fs::remove_file(&overlay_path);
		}

		let settings = result.expect("default settings should load and merge");
		let database = settings
			.database
			.postgres()
			.expect("expected postgres settings");

		assert_eq!(settings.environment, "development");
		assert_eq!(database.host, "localhost");
		assert_eq!(database.user, "override-user");
		assert_eq!(database.max_connections, 50);
		assert_eq!(database.port, 5432);
	}

	#[test]
	fn default_file_without_environment_uses_local_overlay_success() {
		let _guard = cwd_test_lock();
		let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		let base_path = crate_root.join(DEFAULT_SETTINGS_FILE);
		let local_overlay_path = crate_root.join("settings.local.toml");
		let development_overlay_path = crate_root.join("settings.development.toml");

		let base_backup = fs::read_to_string(&base_path).ok();
		let local_overlay_backup = fs::read_to_string(&local_overlay_path).ok();
		let development_overlay_backup = fs::read_to_string(&development_overlay_path).ok();

		let base_content = r#"
database_type = "postgres"

[database]
host = "localhost"
port = 5432
user = "base-user"
name = "appdb"
password_file = "test-utils/unit-test-resources/secrets/db_password_with_newline.txt"
max_connections = 10
connect_timeout_secs = 5
ssl_mode = "disable"
"#;

		let local_overlay_content = r#"
[database]
user = "local-user"
"#;

		let development_overlay_content = r#"
[database]
user = "development-user"
"#;

		let password_file = settings_file_path("test-utils/unit-test-resources/secrets/db_password_with_newline.txt")
			.to_string_lossy()
			.replace('\\', "\\\\");
		let base_content = base_content.replace(
			"test-utils/unit-test-resources/secrets/db_password_with_newline.txt",
			&password_file,
		);

		fs::write(&base_path, base_content).expect("base settings file should be written");
		fs::write(&local_overlay_path, local_overlay_content)
			.expect("local overlay settings file should be written");
		fs::write(&development_overlay_path, development_overlay_content)
			.expect("development overlay settings file should be written");

		let result = AppSettings::from_default_file();

		if let Some(content) = base_backup {
			fs::write(&base_path, content).expect("base settings backup should be restored");
		} else {
			let _ = fs::remove_file(&base_path);
		}

		if let Some(content) = local_overlay_backup {
			fs::write(&local_overlay_path, content)
				.expect("local overlay settings backup should be restored");
		} else {
			let _ = fs::remove_file(&local_overlay_path);
		}

		if let Some(content) = development_overlay_backup {
			fs::write(&development_overlay_path, content)
				.expect("development overlay settings backup should be restored");
		} else {
			let _ = fs::remove_file(&development_overlay_path);
		}

		let settings = result.expect("default settings should load and merge");
		let database = settings
			.database
			.postgres()
			.expect("expected postgres settings");

		assert_eq!(settings.environment, "local");
		assert_eq!(database.user, "local-user");
	}

	#[test]
	fn missing_default_settings_file_failure() {
		let _guard = cwd_test_lock();
		let temp_dir = unique_temp_dir("settings-missing");
		fs::create_dir_all(&temp_dir).expect("temp directory should be created");

		let original_cwd = std::env::current_dir().expect("current directory should be readable");
		std::env::set_current_dir(&temp_dir).expect("should switch to temp directory");

		let result = AppSettings::from_default_file();

		std::env::set_current_dir(original_cwd).expect("should restore original working directory");
		fs::remove_dir_all(temp_dir).expect("temp directory should be removed");

		assert!(matches!(result, Err(AppSettingsError::Io(_))));
	}

	#[test]
	fn database_password_file_loading_success() {
		let password_file = fixture_path("secrets/db_password_with_newline.txt");
		let settings = AppSettings {
			environment: "development".to_owned(),
			database: DatabaseSettings::Postgres(PostgresSettings {
				host: "localhost".to_owned(),
				port: 5432,
				user: "appuser".to_owned(),
				name: "appdb".to_owned(),
				password_file: Some(password_file.to_string_lossy().to_string()),
				max_connections: 10,
				connect_timeout_secs: 5,
				ssl_mode: "disable".to_owned(),
			}),
		};

		let password = settings.get_database_password().expect("expected password file to load");

		assert_eq!(password.expose_secret(), "super-secret-password");
	}

	#[test]
	fn missing_database_password_source_failure() {
		let settings = AppSettings {
			environment: "development".to_owned(),
			database: DatabaseSettings::Postgres(PostgresSettings {
				host: "localhost".to_owned(),
				port: 5432,
				user: "appuser".to_owned(),
				name: "appdb".to_owned(),
				password_file: None,
				max_connections: 10,
				connect_timeout_secs: 5,
				ssl_mode: "disable".to_owned(),
			}),
		};

		let result = settings.get_database_password();

		assert!(matches!(result, Err(AppSettingsError::MissingPasswordSource)));
	}
}

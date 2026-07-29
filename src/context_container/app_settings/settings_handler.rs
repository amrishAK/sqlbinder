use std::fs;
use std::path::Path;

use secrecy::SecretString;

use super::error::AppSettingsError;
use super::model::{
	DEFAULT_SETTINGS_FILE,
	AppSettings,
};

use super::secret_handler::load_secret_from_file;

impl AppSettings {

	pub fn load_settings() -> Result<AppSettings, AppSettingsError> {
		Self::from_default_file()
	}

	/// Recursively merge TOML values, replacing scalars and deep-merging tables.
	fn merge_toml_values(base: &mut toml::Value, overlay: &toml::Value) {
		match (base, overlay) {
			(toml::Value::Table(base_table), toml::Value::Table(overlay_table)) => {
				for (key, overlay_value) in overlay_table {
					if let Some(base_value) = base_table.get_mut(key) {
						Self::merge_toml_values(base_value, overlay_value);
					} else {
						base_table.insert(key.clone(), overlay_value.clone());
					}
				}
			}
			(base_value, overlay_value) => {
				*base_value = overlay_value.clone();
			}
		}
	}

	/// Load base settings and optionally merge an environment-specific overlay file.
	fn from_default_file() -> Result<Self, AppSettingsError> {
		// Load base settings from settings.toml
		let base_raw = fs::read_to_string(DEFAULT_SETTINGS_FILE)?;
		let mut base_value: toml::Value = toml::from_str(&base_raw)?;

		// Extract environment from base settings
		let base_settings: AppSettings = toml::from_str(&base_raw)?;
		let env_specific_file = format!("settings.{}.toml", base_settings.environment);

		// Try to load and merge environment-specific settings
		if Path::new(&env_specific_file).exists() {
			let env_raw = fs::read_to_string(&env_specific_file)?;
			let env_value: toml::Value = toml::from_str(&env_raw)?;

			// Merge environment-specific values into base, including nested tables.
			Self::merge_toml_values(&mut base_value, &env_value);
		}

		// Deserialize merged value without string round-trip.
		base_value.try_into().map_err(Into::into)
	}

	/// Load and deserialize settings from a specific TOML file.
	#[cfg(test)]
	fn from_file(path: impl AsRef<Path>) -> Result<Self, AppSettingsError> {
		let raw = fs::read_to_string(path)?;
		toml::from_str(&raw).map_err(Into::into)
	}

	/// Load the database password from the configured password file.
	pub fn get_database_password(&self) -> Result<SecretString, AppSettingsError> {
		let path = self
			.database
			.password_file
			.as_deref()
			.ok_or(AppSettingsError::MissingPasswordSource)?;

		load_secret_from_file(path)
	}
}

#[cfg(test)]
mod tests {
	use std::fs;
	use std::path::PathBuf;
	use std::sync::{Mutex, OnceLock};
	use std::time::{SystemTime, UNIX_EPOCH};

	use secrecy::ExposeSecret;

	use super::*;
	use crate::context_container::DatabaseSettings;

	fn fixture_path(relative: &str) -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("test-utils")
			.join("unit-test-resources")
			.join(relative)
	}

	fn cwd_test_lock() -> &'static Mutex<()> {
		static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
		LOCK.get_or_init(|| Mutex::new(()))
	}

	fn unique_temp_dir(prefix: &str) -> PathBuf {
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("system time should be after unix epoch")
			.as_nanos();
		std::env::temp_dir().join(format!("sqlbinder-{prefix}-{now}"))
	}

	#[test]
	fn valid_settings_file_parsing_success() {
		let settings_file = fixture_path("settings/base_valid.toml");

		let settings = AppSettings::from_file(&settings_file).expect("expected valid settings fixture");

		assert_eq!(settings.environment, "development");
		assert_eq!(settings.database.host, "localhost");
		assert_eq!(settings.database.port, 5432);
		assert_eq!(settings.database.user, "appuser");
		assert_eq!(settings.database.name, "appdb");
		assert_eq!(settings.database.max_connections, 10);
		assert_eq!(settings.database.connect_timeout_secs, 5);
		assert_eq!(settings.database.ssl_mode, "disable");
	}

	#[test]
	fn missing_environment_defaults_to_development_success() {
		let settings_file = fixture_path("settings/base_without_environment.toml");

		let settings = AppSettings::from_file(&settings_file).expect("expected valid settings fixture");

		assert_eq!(settings.environment, "development");
	}

	#[test]
	fn invalid_toml_settings_file_failure() {
		let settings_file = fixture_path("settings/invalid.toml");

		let result = AppSettings::from_file(&settings_file);

		assert!(matches!(result, Err(AppSettingsError::ParseToml(_))));
	}

	#[test]
	fn default_file_and_environment_override_merge_success() {
		let _guard = cwd_test_lock().lock().expect("cwd test lock should be available");
		let temp_dir = unique_temp_dir("settings-merge");
		fs::create_dir_all(&temp_dir).expect("temp directory should be created");

		let base_fixture = fixture_path("settings/default_base.toml");
		let env_fixture = fixture_path("settings/default_development_override.toml");
		let base_content = fs::read_to_string(&base_fixture).expect("base fixture should be readable");
		let env_content = fs::read_to_string(&env_fixture).expect("env fixture should be readable");

		fs::write(temp_dir.join("settings.toml"), base_content)
			.expect("base temp settings file should be written");
		fs::write(temp_dir.join("settings.development.toml"), env_content)
			.expect("env temp settings file should be written");

		let original_cwd = std::env::current_dir().expect("current directory should be readable");
		std::env::set_current_dir(&temp_dir).expect("should switch to temp directory");

		let settings = AppSettings::from_default_file().expect("default settings should load and merge");

		std::env::set_current_dir(original_cwd).expect("should restore original working directory");
		fs::remove_dir_all(temp_dir).expect("temp directory should be removed");

		assert_eq!(settings.environment, "development");
		assert_eq!(settings.database.host, "localhost");
		assert_eq!(settings.database.user, "override-user");
		assert_eq!(settings.database.max_connections, 50);
		assert_eq!(settings.database.port, 5432);
	}

	#[test]
	fn missing_default_settings_file_failure() {
		let _guard = cwd_test_lock().lock().expect("cwd test lock should be available");
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
			database: DatabaseSettings {
				host: "localhost".to_owned(),
				port: 5432,
				user: "appuser".to_owned(),
				name: "appdb".to_owned(),
				password_file: Some(password_file.to_string_lossy().to_string()),
				max_connections: 10,
				connect_timeout_secs: 5,
				ssl_mode: "disable".to_owned(),
			},
		};

		let password = settings.get_database_password().expect("expected password file to load");

		assert_eq!(password.expose_secret(), "super-secret-password");
	}

	#[test]
	fn missing_database_password_source_failure() {
		let settings = AppSettings {
			environment: "development".to_owned(),
			database: DatabaseSettings {
				host: "localhost".to_owned(),
				port: 5432,
				user: "appuser".to_owned(),
				name: "appdb".to_owned(),
				password_file: None,
				max_connections: 10,
				connect_timeout_secs: 5,
				ssl_mode: "disable".to_owned(),
			},
		};

		let result = settings.get_database_password();

		assert!(matches!(result, Err(AppSettingsError::MissingPasswordSource)));
	}
}

use std::fs;
use std::path::Path;

use secrecy::{ExposeSecret, SecretString};

use super::error::AppSettingsError;

pub(crate) fn load_secret_from_file(path: impl AsRef<Path>) -> Result<SecretString, AppSettingsError> {
	let content = fs::read_to_string(path)?;
	// Intentionally normalize secret files that end with line breaks/whitespace.
	Ok(SecretString::from(content.trim_end().to_owned()))
}

pub(crate) fn validate_secret_file(path: impl AsRef<Path>) -> Result<(), AppSettingsError> {
	let secret = load_secret_from_file(path)?;
	if secret.expose_secret().is_empty() {
		return Err(AppSettingsError::InvalidConfig(
			"Secret file cannot be empty".to_string(),
		));
	}
	Ok(())
}

pub(crate) fn validate_required_secret_file(path: Option<&str>) -> Result<(), AppSettingsError> {
	let path = path.ok_or_else(|| {
		AppSettingsError::InvalidConfig(
			"Database password_file is required but not provided".to_string(),
		)
	})?;

	validate_secret_file(path)
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use secrecy::ExposeSecret;

	use super::*;

	fn fixture_path(relative: &str) -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("test-utils")
			.join("unit-test-resources")
			.join(relative)
	}

	#[test]
	fn secret_file_trailing_whitespace_trim_success() {
		let secret_file = fixture_path("secrets/db_password_with_newline.txt");

		let secret = load_secret_from_file(&secret_file).expect("expected fixture secret file to load");

		assert_eq!(secret.expose_secret(), "super-secret-password");
	}

	#[test]
	fn secret_file_missing_failure() {
		let missing_file = fixture_path("secrets/does_not_exist.txt");

		let result = load_secret_from_file(&missing_file);

		assert!(matches!(result, Err(AppSettingsError::Io(_))));
	}
}
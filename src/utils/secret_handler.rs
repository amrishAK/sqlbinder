use std::fs;
use std::path::Path;

use secrecy::SecretString;

use super::error::SettingsError;

pub(crate) fn load_secret_from_file(path: impl AsRef<Path>) -> Result<SecretString, SettingsError> {
	let content = fs::read_to_string(path)?;
	// Intentionally normalize secret files that end with line breaks/whitespace.
	Ok(SecretString::from(content.trim_end().to_owned()))
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

		assert!(matches!(result, Err(SettingsError::Io(_))));
	}
}
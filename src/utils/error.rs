use std::io;
use std::num::{ParseFloatError, ParseIntError};

use thiserror::Error;

/// Enumerates failures that can occur while loading and parsing application settings.
#[derive(Debug, Error)]
pub enum AppSettingsError {
	#[error("I/O error: {0}")]
	Io(#[from] io::Error),
	#[error("TOML parsing error: {0}")]
	ParseToml(#[from] toml::de::Error),
	#[error("integer parsing error: {0}")]
	ParseInt(#[from] ParseIntError),
	#[error("float parsing error: {0}")]
	ParseFloat(#[from] ParseFloatError),
	#[error("no password source configured (password_file)")]
	MissingPasswordSource,
	#[error("required environment variable is missing: {0}")]
	MissingEnvVar(String),
	#[error("settings have not been initialized")]
	SettingsNotInitialized,
	#[error("settings have already been initialized")]
	SettingsAlreadyInitialized,
}

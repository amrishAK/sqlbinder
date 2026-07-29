use thiserror::Error;
use super::app_settings::AppSettingsError;

/// Errors returned by context container initialization and lifecycle operations.
#[derive(Debug, Error)]
pub enum ContextContainerError {
	#[error("failed to load application settings: {0}")]
	Settings(#[from] AppSettingsError),
}

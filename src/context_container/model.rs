use super::app_settings::AppSettings;
use std::sync::Arc;


/// Application context with settings loaded at creation time.
#[derive(Debug, Clone)]
pub struct ContextContainer {
	pub(crate) app_settings: Arc<AppSettings>,
}
use crate::utils::AppSettings;
use std::sync::Arc;


/// Application context that may be initialized in a startup phase.
#[derive(Debug, Clone)]
pub struct ContextContainer {
	pub(crate) app_settings: Option<Arc<AppSettings>>,
}
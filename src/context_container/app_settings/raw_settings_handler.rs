use std::fs;
use std::path::PathBuf;
use super::model::DEFAULT_SETTINGS_FILE;

use super::error::AppSettingsError;


fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn default_settings_path() -> PathBuf {
    project_root().join(DEFAULT_SETTINGS_FILE)
}

/// Load base settings and optionally merge an environment-specific overlay file.
pub(crate) fn from_default_file() -> Result<toml::Value, AppSettingsError> {
    let base_path = default_settings_path();
    let base_raw = fs::read_to_string(&base_path)?;
    let mut base_value: toml::Value = toml::from_str(&base_raw)?;

    let environment = base_value
        .get("environment")
        .and_then(toml::Value::as_str)
        .unwrap_or("local")
        .to_owned();
    let env_specific_file = project_root().join(format!("settings.{environment}.toml"));

    if env_specific_file.exists() {
        let env_raw = fs::read_to_string(&env_specific_file)?;
        let env_value: toml::Value = toml::from_str(&env_raw)?;
        merge_toml_values(&mut base_value, &env_value);
    }
    
    Ok(base_value)
}

/// Recursively merge TOML values, replacing scalars and deep-merging tables.
fn merge_toml_values(base: &mut toml::Value, overlay: &toml::Value) {
match (base, overlay) {
    (toml::Value::Table(base_table), toml::Value::Table(overlay_table)) => {
        for (key, overlay_value) in overlay_table {
            if let Some(base_value) = base_table.get_mut(key) {
                merge_toml_values(base_value, overlay_value);
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






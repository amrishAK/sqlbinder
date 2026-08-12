use serde::de::Error as _;

use super::error::AppSettingsError;

pub(super) fn required_string(
	table: &toml::Table,
	field_name: &str,
	error_message: &str,
) -> Result<String, AppSettingsError> {
	let value = table
		.get(field_name)
		.and_then(toml::Value::as_str)
		.map(str::to_owned)
		.ok_or_else(|| {
			AppSettingsError::ParseToml(toml::de::Error::custom(error_message.to_owned()))
		})?;

	Ok(value)
}

pub(super) fn optional_string(table: &toml::Table, field_name: &str) -> Option<String> {
	table
		.get(field_name)
		.and_then(toml::Value::as_str)
		.map(str::to_owned)
}


pub(super) fn required_u16(
	table: &toml::Table,
	field_name: &str,
	error_message: &str,
) -> Result<u16, AppSettingsError> {
	let number = table
		.get(field_name)
		.and_then(toml::Value::as_integer)
		.ok_or_else(|| {
			AppSettingsError::ParseToml(toml::de::Error::custom(error_message.to_owned()))
		})?;

	u16::try_from(number).map_err(|_| {
		AppSettingsError::ParseToml(toml::de::Error::custom(error_message.to_owned()))
	})
}

pub(super) fn required_u32(
	table: &toml::Table,
	field_name: &str,
	error_message: &str,
) -> Result<u32, AppSettingsError> {
	let number = table
		.get(field_name)
		.and_then(toml::Value::as_integer)
		.ok_or_else(|| {
			AppSettingsError::ParseToml(toml::de::Error::custom(error_message.to_owned()))
		})?;

	u32::try_from(number).map_err(|_| {
		AppSettingsError::ParseToml(toml::de::Error::custom(error_message.to_owned()))
	})
}

pub(super) fn required_u64(
	table: &toml::Table,
	field_name: &str,
	error_message: &str,
) -> Result<u64, AppSettingsError> {
	let number = table
		.get(field_name)
		.and_then(toml::Value::as_integer)
		.ok_or_else(|| {
			AppSettingsError::ParseToml(toml::de::Error::custom(error_message.to_owned()))
		})?;

	u64::try_from(number).map_err(|_| {
		AppSettingsError::ParseToml(toml::de::Error::custom(error_message.to_owned()))
	})
}
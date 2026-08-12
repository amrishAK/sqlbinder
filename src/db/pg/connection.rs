use std::time::Duration;

use secrecy::ExposeSecret;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

use crate::context_container::app_settings::model::PostgresSettings;
use crate::context_container::app_settings::secret_handler;

use super::model::PostgresPool;
use crate::db::error::DbConnectionError;

pub(crate) fn get_pg_pool(database_settings: &PostgresSettings) -> Result<PostgresPool, DbConnectionError> {
    let options = build_connect_options(database_settings)?;
    let pool = PgPoolOptions::new()
        .max_connections(database_settings.max_connections)
        .acquire_timeout(Duration::from_secs(database_settings.connect_timeout_secs))
        .connect_lazy_with(options);

    Ok(PostgresPool::new(pool))
}

fn build_connect_options(settings: &PostgresSettings) -> Result<PgConnectOptions, DbConnectionError> {
    let ssl_mode = parse_ssl_mode(&settings.ssl_mode)?;
    let password = settings
        .password_file
        .as_deref()
        .map(secret_handler::load_secret_from_file)
        .transpose()?
        .ok_or(DbConnectionError::MissingPasswordSource { backend: "postgres" })?;

    Ok(PgConnectOptions::new()
        .host(&settings.host)
        .port(settings.port)
        .username(&settings.user)
        .password(password.expose_secret())
        .database(&settings.name)
        .ssl_mode(ssl_mode))
}

fn parse_ssl_mode(value: &str) -> Result<PgSslMode, DbConnectionError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "disable" | "disabled" => Ok(PgSslMode::Disable),
        "allow" => Ok(PgSslMode::Allow),
        "prefer" => Ok(PgSslMode::Prefer),
        "require" | "required" | "enabled" => Ok(PgSslMode::Require),
        "verify-ca" => Ok(PgSslMode::VerifyCa),
        "verify-full" => Ok(PgSslMode::VerifyFull),
        other => Err(DbConnectionError::UnsupportedPostgresSslMode(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn sample_settings(password_file: Option<String>) -> PostgresSettings {
        PostgresSettings {
            host: "localhost".to_owned(),
            port: 5432,
            user: "postgres".to_owned(),
            name: "sqlbinder".to_owned(),
            password_file,
            max_connections: 5,
            connect_timeout_secs: 10,
            ssl_mode: "prefer".to_owned(),
        }
    }

    fn temp_secret_file() -> PathBuf {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("sqlbinder-secret-{unique_id}.txt"));
        fs::write(&path, "super-secret-password").expect("expected temporary secret file to be written");
        path
    }

    #[test]
    fn parse_ssl_mode_accepts_supported_aliases_success() {
        let cases = [
            ("disable", PgSslMode::Disable),
            (" disabled ", PgSslMode::Disable),
            ("ALLOW", PgSslMode::Allow),
            ("Prefer", PgSslMode::Prefer),
            ("required", PgSslMode::Require),
            (" enabled ", PgSslMode::Require),
            ("verify-ca", PgSslMode::VerifyCa),
            ("VERIFY-FULL", PgSslMode::VerifyFull),
        ];

        for (input, expected) in cases {
            let parsed = parse_ssl_mode(input).expect("expected supported ssl mode to parse");
            assert_eq!(format!("{:?}", parsed), format!("{:?}", expected));
        }
    }

    #[test]
    fn parse_ssl_mode_rejects_unknown_value_failure() {
        let error = parse_ssl_mode("not-a-real-mode").unwrap_err();

        assert!(matches!(error, DbConnectionError::UnsupportedPostgresSslMode(value) if value == "not-a-real-mode"));
    }

    #[test]
    fn build_connect_options_requires_password_file_failure() {
        let settings = sample_settings(None);

        let error = build_connect_options(&settings).unwrap_err();

        assert!(matches!(error, DbConnectionError::MissingPasswordSource { backend } if backend == "postgres"));
    }

    #[test]
    fn build_connect_options_uses_password_file_success() {
        let path = temp_secret_file();
        let settings = sample_settings(Some(path.to_string_lossy().into_owned()));

        let options = build_connect_options(&settings).expect("expected password file to be loaded");

        assert_eq!(options.get_host(), "localhost");
        assert_eq!(options.get_database(), Some("sqlbinder"));

        let _ = fs::remove_file(path);
    }
}
//! Loads runtime configuration from environment variables for AppState.
//! Used by AppState::new to construct service URLs, database settings, etc.

use std::env;

use secrecy::SecretBox;
use sha2::{Digest, Sha256};

use crate::{AppError, constants::EVENT_ENCRYPTION_KEY_LEN};

const DEFAULT_DATABASE_URL: &str = "postgres://eks@localhost/eks";
const DEFAULT_EVENT_ENCRYPTION_KEY: &str = "dev-only-event-encryption-key";

#[derive(Debug)]
pub struct Config {
    pub database_url: &'static str,
    pub event_encryption_key: SecretBox<Vec<u8>>,
}

/// Helper function to get environment variable or return an error
pub fn get_env(name: &'static str, _dev_default: &'static str) -> Result<String, AppError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        #[cfg(feature = "dev-features")]
        Err(_) => Ok(_dev_default.to_string()),
        #[cfg(not(feature = "dev-features"))]
        Err(_) => Err(AppError::MissingEnvVar(name)),
    }
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Self::from_env_with(get_env)
    }

    pub fn from_env_with<F>(get: F) -> Result<Self, AppError>
    where
        F: Fn(&'static str, &'static str) -> Result<String, AppError>,
    {
        let event_encryption_key = derive_event_encryption_key(get(
            "EVENT_ENCRYPTION_KEY",
            DEFAULT_EVENT_ENCRYPTION_KEY,
        )?)?;

        Ok(Self {
            database_url: Box::leak(get("DATABASE_URL", DEFAULT_DATABASE_URL)?.into_boxed_str()),
            event_encryption_key,
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            database_url: DEFAULT_DATABASE_URL,
            event_encryption_key: derive_event_encryption_key(
                DEFAULT_EVENT_ENCRYPTION_KEY.to_string(),
            )
            .expect("test encryption key"),
        }
    }
}

fn derive_event_encryption_key(value: String) -> Result<SecretBox<Vec<u8>>, AppError> {
    if value.trim().is_empty() {
        return Err(AppError::ConfigLoadError(
            "EVENT_ENCRYPTION_KEY must not be empty".to_string(),
        ));
    }

    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let key = digest[..EVENT_ENCRYPTION_KEY_LEN].to_vec();

    Ok(SecretBox::new(Box::new(key)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_database_url_from_provider() {
        let config = Config::from_env_with(|key, _default| match key {
            "DATABASE_URL" => Ok("postgres://example".to_string()),
            "EVENT_ENCRYPTION_KEY" => Ok("unit-test-key".to_string()),
            _ => Err(AppError::MissingEnvVar(key)),
        })
        .unwrap();

        assert_eq!(config.database_url, "postgres://example");
    }

    #[test]
    fn returns_error_when_env_missing() {
        let key: &'static str = "EVENT_ENCRYPTION_KEY";

        let err =
            Config::from_env_with(|_, _default| Err(AppError::MissingEnvVar(key))).unwrap_err();
        match err {
            AppError::MissingEnvVar(var) => assert_eq!(var, key),
            _ => panic!("unexpected error: {err:?}"),
        }
    }
}

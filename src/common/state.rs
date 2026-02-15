//! Application state container and request extractors.
//! Holds, among others: configuration, database pool, and CSRF tokens for handlers.

use axum::{
    extract::{FromRef},
};
use std::sync::Arc;

use crate::{AppError, AppStore, Config, CsrfTokens, store::EventCrypto};

#[derive(FromRef, Clone)]
pub struct AppState {
    pub store: AppStore,
    pub csrf_tokens: CsrfTokens,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let config = Config::from_env()?;
        // let pool = PgPool::connect_lazy(config.database_url)?;
        let csrf_tokens = CsrfTokens::default();
        let event_crypto = Arc::new(EventCrypto::new(config.event_encryption_key)?);
        let store = AppStore::new_filesystem("./data".into(), event_crypto);

        Ok(Self {
            store,
            csrf_tokens,
        })
    }

    #[cfg(test)]
    pub async fn new_for_tests() -> Self {
        Self {
            store: AppStore::new_for_test().await,
            csrf_tokens: CsrfTokens::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[tokio::test]
    async fn new_for_tests_sets_tokens() -> Result<(), sqlx::Error> {
        let state = AppState::new_for_tests().await;
        let token = state.csrf_tokens.issue();
        assert!(state.csrf_tokens.consume(&token.value));

        Ok(())
    }
}

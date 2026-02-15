use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
    AppError,
};

mod crypto;
mod database;
mod event;
mod filesystem;
mod getters;
mod persistence;
mod reducer;

pub use event::{AppEvent, StoreEvent};
pub use crypto::EventCrypto;

#[cfg(test)]
mod tests;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppStoreData {
    political_group: PoliticalGroup,
    persons: HashMap<PersonId, Person>,
    candidate_lists: HashMap<CandidateListId, CandidateList>,
    authorised_agents: HashMap<AuthorisedAgentId, AuthorisedAgent>,
    list_submitters: HashMap<ListSubmitterId, ListSubmitter>,
    substitute_submitters: HashMap<SubstituteSubmitterId, SubstituteSubmitter>,
    // Track the last event ID applied to this store instance for synchronization purposes
    last_event_id: usize,
}

#[derive(Clone)]
pub enum AppStorePersistence {
    Database(sqlx::PgPool),
    Filesystem(std::path::PathBuf),
    None,
}

#[derive(Clone)]
pub struct AppStore {
    pub persistence: AppStorePersistence,
    crypto: Arc<EventCrypto>,
    data: Arc<parking_lot::RwLock<AppStoreData>>,
}

impl AppStore {
    pub fn new(pool: sqlx::PgPool, crypto: Arc<EventCrypto>) -> Self {
        AppStore {
            persistence: AppStorePersistence::Database(pool),
            crypto,
            data: Default::default(),
        }
    }

    pub fn new_filesystem(path: std::path::PathBuf, crypto: Arc<EventCrypto>) -> Self {
        AppStore {
            persistence: AppStorePersistence::Filesystem(path),
            crypto,
            data: Default::default(),
        }
    }

    #[cfg(test)]
    pub async fn new_for_test() -> Self {
        let political_group_id = crate::political_groups::PoliticalGroupId::new();
        let political_group = crate::test_utils::sample_political_group(political_group_id);

        let store = AppStore {
            persistence: AppStorePersistence::None,
            crypto: test_event_crypto(),
            data: Default::default(),
        };

        political_group.update(&store).await.expect("store update");

        store
    }

    fn encrypt_event_payload(&self, payload: &AppEvent) -> Result<Vec<u8>, AppError> {
        let serialized =
            postcard::to_stdvec(payload).map_err(|_| AppError::InternalServerError)?;
        self.crypto.encrypt(&serialized)
    }

    fn decrypt_event_payload(&self, payload: &[u8]) -> Result<AppEvent, AppError> {
        let decrypted = self.crypto.decrypt(payload)?;
        postcard::from_bytes(&decrypted).map_err(|_| AppError::InternalServerError)
    }
}

#[cfg(test)]
pub fn test_event_crypto() -> Arc<EventCrypto> {
    use secrecy::SecretBox;

    Arc::new(
        EventCrypto::new(SecretBox::new(Box::new(vec![0u8; crate::constants::EVENT_ENCRYPTION_KEY_LEN])))
            .expect("test crypto"),
    )
}

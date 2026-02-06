use crate::{AppError, AppEvent, AppStore, id_newtype};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct PoliticalGroupId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalGroup {
    pub id: PoliticalGroupId,

    pub long_list_allowed: Option<bool>,
    pub legal_name: Option<String>,
    pub display_name: Option<String>,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,

    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

impl PoliticalGroup {
    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdatePoliticalGroup(self.clone()))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdatePoliticalGroup(self.clone()))
            .await
    }
}

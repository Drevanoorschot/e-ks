use crate::{
    AppError, AppStore, common::store::AppEvent, id_newtype, political_groups::PoliticalGroupId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,

    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,
    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

impl AuthorisedAgent {
    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{prefix} {}", self.last_name)
        } else {
            self.last_name.clone()
        }
    }

    pub async fn create(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<AuthorisedAgent, AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        let now = Utc::now();
        let authorised_agent = AuthorisedAgent {
            created_at: now,
            updated_at: now,
            ..self.clone()
        };

        store
            .update(AppEvent::CreateAuthorisedAgent(authorised_agent.clone()))
            .await?;

        Ok(authorised_agent)
    }

    pub async fn update(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<AuthorisedAgent, AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        let existing = store
            .get_authorised_agent(self.id)?
            .ok_or(AppError::GenericNotFound)?;
        let updated = AuthorisedAgent {
            created_at: existing.created_at,
            updated_at: Utc::now(),
            ..self.clone()
        };

        store
            .update(AppEvent::UpdateAuthorisedAgent(updated.clone()))
            .await?;

        Ok(updated)
    }

    pub async fn delete(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<(), AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store
            .update(AppEvent::DeleteAuthorisedAgent(self.id))
            .await?;

        Ok(())
    }

    pub async fn delete_by_id(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
        authorised_agent_id: AuthorisedAgentId,
    ) -> Result<(), AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store
            .update(AppEvent::DeleteAuthorisedAgent(authorised_agent_id))
            .await?;

        Ok(())
    }
}

use crate::{
    AppError, AppEvent, AppStore, id_newtype,
    political_groups::{
        AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, SubstituteSubmitter,
        SubstituteSubmitterId,
    },
};
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
    pub fn get_single(store: &AppStore) -> Result<Option<PoliticalGroup>, AppError> {
        let political_group = store.get_political_group()?;

        Ok(Some(political_group))
    }

    pub fn list_submitters(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<Vec<ListSubmitter>, AppError> {
        let political_group = PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store.get_list_submitters()
    }

    pub fn list_authorised_agents(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<Vec<AuthorisedAgent>, AppError> {
        let political_group = PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store.get_authorised_agents()
    }

    pub fn list_substitute_submitters(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<Vec<SubstituteSubmitter>, AppError> {
        let political_group = PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store.get_substitute_submitters()
    }

    pub fn get_authorised_agent(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
        authorised_agent_id: AuthorisedAgentId,
    ) -> Result<AuthorisedAgent, AppError> {
        let political_group = PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store
            .get_authorised_agent(authorised_agent_id)?
            .ok_or(AppError::GenericNotFound)
    }

    pub fn get_list_submitter(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
        submitter_id: ListSubmitterId,
    ) -> Result<ListSubmitter, AppError> {
        let political_group = PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store
            .get_list_submitter(submitter_id)?
            .ok_or(AppError::GenericNotFound)
    }

    pub fn get_substitute_submitter(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
        submitter_id: SubstituteSubmitterId,
    ) -> Result<SubstituteSubmitter, AppError> {
        let political_group = PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store
            .get_substitute_submitter(submitter_id)?
            .ok_or(AppError::GenericNotFound)
    }

    pub async fn create(&self, store: &AppStore) -> Result<PoliticalGroup, AppError> {
        let now = Utc::now();
        let political_group = PoliticalGroup {
            created_at: now,
            updated_at: now,
            ..self.clone()
        };

        store
            .update(AppEvent::UpdatePoliticalGroup(political_group.clone()))
            .await?;

        Ok(political_group)
    }

    pub async fn update(&self, store: &AppStore) -> Result<PoliticalGroup, AppError> {
        let existing = PoliticalGroup::get_single(store)?
            .ok_or_else(|| AppError::NotFound("Political group not found.".to_string()))?;
        let updated = PoliticalGroup {
            created_at: existing.created_at,
            updated_at: Utc::now(),
            ..self.clone()
        };

        store
            .update(AppEvent::UpdatePoliticalGroup(updated.clone()))
            .await?;

        Ok(updated)
    }
}

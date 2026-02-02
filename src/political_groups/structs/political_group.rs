use crate::{
    AppError, AppStore, id_newtype,
    common::store::AppEvent,
    political_groups::{AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct PoliticalGroupId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalGroup {
    pub id: PoliticalGroupId,

    pub long_list_allowed: Option<bool>,

    pub legal_name: String,
    pub legal_name_confirmed: Option<bool>,

    pub display_name: String,
    pub display_name_confirmed: Option<bool>,

    pub authorised_agent_id: Option<AuthorisedAgentId>,
    pub list_submitter_id: Option<ListSubmitterId>,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,

    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

impl PoliticalGroup {
    pub fn get_single(store: &AppStore) -> Result<Option<PoliticalGroup>, AppError> {
        let political_group = store.get_political_group();

        if political_group.legal_name.is_empty()
            && political_group.display_name.is_empty()
            && political_group.long_list_allowed.is_none()
            && political_group.legal_name_confirmed.is_none()
            && political_group.display_name_confirmed.is_none()
            && political_group.authorised_agent_id.is_none()
            && political_group.list_submitter_id.is_none()
        {
            return Ok(None);
        }

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

        Ok(store.get_list_submitters())
    }

    pub fn list_authorised_agents(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<Vec<AuthorisedAgent>, AppError> {
        let political_group = PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        Ok(store.get_authorised_agents())
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

        store.get_list_submitter(submitter_id)
    }

    pub async fn set_default_list_submitter(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
        submitter_id: Option<ListSubmitterId>,
    ) -> Result<(), AppError> {
        let Some(mut political_group) = PoliticalGroup::get_single(store)? else {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        };
        if political_group.id != political_group_id {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        if let Some(id) = submitter_id {
            store.get_list_submitter(id)?;
        }

        political_group.list_submitter_id = submitter_id;
        political_group.updated_at = Utc::now();

        store
            .update(AppEvent::UpdatePoliticalGroup(political_group))
            .await?;

        Ok(())
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

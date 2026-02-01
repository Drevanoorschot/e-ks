use crate::{
    AppError, AppStore,
    common::store::AppEvent,
    political_groups::{
        AuthorisedAgent, ListSubmitter, ListSubmitterId, PoliticalGroup, PoliticalGroupId,
    },
};
use chrono::Utc;

pub fn get_single_political_group(store: &AppStore) -> Result<Option<PoliticalGroup>, AppError> {
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

pub async fn update_political_group(
    store: &AppStore,
    political_group: &PoliticalGroup,
) -> Result<PoliticalGroup, AppError> {
    let existing = get_single_political_group(store)?
        .ok_or_else(|| AppError::NotFound("Political group not found.".to_string()))?;

    let updated = PoliticalGroup {
        created_at: existing.created_at,
        updated_at: Utc::now(),
        ..political_group.clone()
    };

    store
        .update(AppEvent::UpdatePoliticalGroup(updated.clone()))
        .await?;

    Ok(updated)
}

#[cfg(any(test, feature = "fixtures"))]
pub async fn create_political_group(
    store: &AppStore,
    political_group: &PoliticalGroup,
) -> Result<PoliticalGroup, AppError> {
    let now = Utc::now();
    let political_group = PoliticalGroup {
        created_at: now,
        updated_at: now,
        ..political_group.clone()
    };

    store
        .update(AppEvent::UpdatePoliticalGroup(political_group.clone()))
        .await?;

    Ok(political_group)
}

pub async fn get_list_submitters(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
) -> Result<Vec<ListSubmitter>, AppError> {
    let political_group = get_single_political_group(store)?;
    if political_group.map(|group| group.id) != Some(political_group_id) {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    Ok(store.get_list_submitters())
}

pub async fn get_authorised_agents(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
) -> Result<Vec<AuthorisedAgent>, AppError> {
    let political_group = get_single_political_group(store)?;
    if political_group.map(|group| group.id) != Some(political_group_id) {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    Ok(store.get_authorised_agents())
}

#[cfg(any(test, feature = "fixtures"))]
pub async fn create_authorised_agent(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
    authorised_agent: &AuthorisedAgent,
) -> Result<AuthorisedAgent, AppError> {
    let political_group = get_single_political_group(store)?;
    if political_group.map(|group| group.id) != Some(political_group_id) {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    let now = Utc::now();
    let authorised_agent = AuthorisedAgent {
        created_at: now,
        updated_at: now,
        ..authorised_agent.clone()
    };

    store
        .update(AppEvent::CreateAuthorisedAgent(authorised_agent.clone()))
        .await?;

    Ok(authorised_agent)
}

pub async fn get_list_submitter(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
    submitter_id: &ListSubmitterId,
) -> Result<ListSubmitter, AppError> {
    let political_group = get_single_political_group(store)?;
    if political_group.map(|group| group.id) != Some(political_group_id) {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    store
        .get_list_submitters()
        .into_iter()
        .find(|submitter| submitter.id == *submitter_id)
        .ok_or_else(|| AppError::NotFound("List submitter not found.".to_string()))
}

pub async fn set_default_list_submitter(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
    submitter_id: Option<ListSubmitterId>,
) -> Result<(), AppError> {
    let Some(mut political_group) = get_single_political_group(store)? else {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    };
    if political_group.id != political_group_id {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    if let Some(id) = submitter_id {
        let exists = store.get_list_submitters().iter().any(|s| s.id == id);
        if !exists {
            return Err(AppError::NotFound("List submitter not found.".to_string()));
        }
    }

    political_group.list_submitter_id = submitter_id;
    political_group.updated_at = Utc::now();

    store
        .update(AppEvent::UpdatePoliticalGroup(political_group))
        .await?;

    Ok(())
}

pub async fn create_list_submitter(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
    list_submitter: &ListSubmitter,
) -> Result<ListSubmitter, AppError> {
    let political_group = get_single_political_group(store)?;
    if political_group.map(|group| group.id) != Some(political_group_id) {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    let now = Utc::now();
    let list_submitter = ListSubmitter {
        created_at: now,
        updated_at: now,
        ..list_submitter.clone()
    };

    store
        .update(AppEvent::CreateListSubmitter(list_submitter.clone()))
        .await?;

    Ok(list_submitter)
}

pub async fn update_list_submitter(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
    list_submitter: &ListSubmitter,
) -> Result<ListSubmitter, AppError> {
    let political_group = get_single_political_group(store)?;
    if political_group.map(|group| group.id) != Some(political_group_id) {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    let existing = store
        .get_list_submitters()
        .into_iter()
        .find(|submitter| submitter.id == list_submitter.id)
        .ok_or_else(|| AppError::NotFound("List submitter not found.".to_string()))?;

    let updated = ListSubmitter {
        created_at: existing.created_at,
        updated_at: Utc::now(),
        ..list_submitter.clone()
    };

    store
        .update(AppEvent::UpdateListSubmitter(updated.clone()))
        .await?;

    Ok(updated)
}

pub async fn remove_list_submitter(
    store: &AppStore,
    political_group_id: PoliticalGroupId,
    list_submitter_id: ListSubmitterId,
) -> Result<(), AppError> {
    let Some(mut political_group) = get_single_political_group(store)? else {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    };
    if political_group.id != political_group_id {
        return Err(AppError::NotFound("Political group not found.".to_string()));
    }

    if political_group.list_submitter_id == Some(list_submitter_id) {
        political_group.list_submitter_id = None;
        political_group.updated_at = Utc::now();
        store
            .update(AppEvent::UpdatePoliticalGroup(political_group))
            .await?;
    }

    store
        .update(AppEvent::DeleteListSubmitter(list_submitter_id))
        .await?;

    Ok(())
}

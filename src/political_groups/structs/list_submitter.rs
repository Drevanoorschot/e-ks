use crate::{AppError, AppEvent, AppStore, id_newtype, political_groups::PoliticalGroupId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct ListSubmitterId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ListSubmitter {
    pub id: ListSubmitterId,

    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,

    pub locality: Option<String>,
    pub postal_code: Option<String>,
    pub house_number: Option<String>,
    pub house_number_addition: Option<String>,
    pub street_name: Option<String>,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,
    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

impl ListSubmitter {
    pub fn is_address_complete(&self) -> bool {
        self.street_name.is_some()
            && self.house_number.is_some()
            && self.postal_code.is_some()
            && self.locality.is_some()
    }

    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{} {}", prefix, self.last_name)
        } else {
            self.last_name.clone()
        }
    }

    pub async fn create(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<ListSubmitter, AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        let now = Utc::now();
        let list_submitter = ListSubmitter {
            created_at: now,
            updated_at: now,
            ..self.clone()
        };

        store
            .update(AppEvent::CreateListSubmitter(list_submitter.clone()))
            .await?;

        Ok(list_submitter)
    }

    pub async fn update(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<ListSubmitter, AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        let existing = store
            .get_list_submitter(self.id)?
            .ok_or(AppError::GenericNotFound)?;

        let updated = ListSubmitter {
            created_at: existing.created_at,
            updated_at: Utc::now(),
            ..self.clone()
        };

        store
            .update(AppEvent::UpdateListSubmitter(updated.clone()))
            .await?;

        Ok(updated)
    }

    pub async fn delete(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<(), AppError> {
        let Some(political_group) = crate::political_groups::PoliticalGroup::get_single(store)?
        else {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        };
        if political_group.id != political_group_id {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store.update(AppEvent::DeleteListSubmitter(self.id)).await?;

        Ok(())
    }

    pub async fn delete_by_id(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
        list_submitter_id: ListSubmitterId,
    ) -> Result<(), AppError> {
        let Some(political_group) = crate::political_groups::PoliticalGroup::get_single(store)?
        else {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        };
        if political_group.id != political_group_id {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store
            .update(AppEvent::DeleteListSubmitter(list_submitter_id))
            .await?;

        Ok(())
    }
}

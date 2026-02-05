use crate::{AppError, AppEvent, AppStore, id_newtype, political_groups::PoliticalGroupId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct SubstituteSubmitterId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SubstituteSubmitter {
    pub id: SubstituteSubmitterId,

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

impl SubstituteSubmitter {
    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{} {}", prefix, self.last_name)
        } else {
            self.last_name.clone()
        }
    }

    pub fn is_address_complete(&self) -> bool {
        self.street_name.is_some()
            && self.house_number.is_some()
            && self.postal_code.is_some()
            && self.locality.is_some()
    }

    pub async fn create(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<SubstituteSubmitter, AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        let now = Utc::now();
        let substitute_submitter = SubstituteSubmitter {
            created_at: now,
            updated_at: now,
            ..self.clone()
        };

        store
            .update(AppEvent::CreateSubstituteSubmitter(
                substitute_submitter.clone(),
            ))
            .await?;

        Ok(substitute_submitter)
    }

    pub async fn update(
        &self,
        store: &AppStore,
        political_group_id: PoliticalGroupId,
    ) -> Result<SubstituteSubmitter, AppError> {
        let political_group = crate::political_groups::PoliticalGroup::get_single(store)?;
        if political_group.map(|group| group.id) != Some(political_group_id) {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        let existing = store
            .get_substitute_submitter(self.id)?
            .ok_or(AppError::GenericNotFound)?;

        let updated = SubstituteSubmitter {
            created_at: existing.created_at,
            updated_at: Utc::now(),
            ..self.clone()
        };

        store
            .update(AppEvent::UpdateSubstituteSubmitter(updated.clone()))
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

        store
            .update(AppEvent::DeleteSubstituteSubmitter(self.id))
            .await?;

        Ok(())
    }

    pub async fn delete_by_id(
        store: &AppStore,
        political_group_id: PoliticalGroupId,
        substitute_submitter_id: SubstituteSubmitterId,
    ) -> Result<(), AppError> {
        let Some(political_group) = crate::political_groups::PoliticalGroup::get_single(store)?
        else {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        };
        if political_group.id != political_group_id {
            return Err(AppError::NotFound("Political group not found.".to_string()));
        }

        store
            .update(AppEvent::DeleteSubstituteSubmitter(substitute_submitter_id))
            .await?;

        Ok(())
    }
}

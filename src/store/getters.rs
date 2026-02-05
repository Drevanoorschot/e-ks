use crate::{
    AppError,
    candidate_lists::{CandidateList, CandidateListId},
    persons::{Person, PersonId},
    political_groups::{
        AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup,
        SubstituteSubmitter, SubstituteSubmitterId,
    },
};

use super::AppStore;

impl AppStore {
    pub fn get_candidate_lists(&self) -> Result<Vec<CandidateList>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.candidate_lists.values().cloned().collect())
    }

    pub fn get_political_group(&self) -> Result<PoliticalGroup, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.political_group.clone())
    }

    pub fn get_persons(&self) -> Result<Vec<Person>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.persons.values().cloned().collect())
    }

    pub fn get_authorised_agents(&self) -> Result<Vec<AuthorisedAgent>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.authorised_agents.values().cloned().collect())
    }

    pub fn get_list_submitters(&self) -> Result<Vec<ListSubmitter>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.list_submitters.values().cloned().collect())
    }

    pub fn get_substitute_submitters(&self) -> Result<Vec<SubstituteSubmitter>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.substitute_submitters.values().cloned().collect())
    }

    pub fn get_person_count(&self) -> Result<usize, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.persons.len())
    }

    pub fn get_candidate_list(
        &self,
        list_id: CandidateListId,
    ) -> Result<Option<CandidateList>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.candidate_lists.get(&list_id).cloned())
    }

    pub fn get_person(&self, person_id: PersonId) -> Result<Option<Person>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.persons.get(&person_id).cloned())
    }

    pub fn get_authorised_agent(
        &self,
        authorised_agent_id: AuthorisedAgentId,
    ) -> Result<Option<AuthorisedAgent>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.authorised_agents.get(&authorised_agent_id).cloned())
    }

    pub fn get_list_submitter(
        &self,
        list_submitter_id: ListSubmitterId,
    ) -> Result<Option<ListSubmitter>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.list_submitters.get(&list_submitter_id).cloned())
    }

    pub fn get_substitute_submitter(
        &self,
        substitute_submitter_id: SubstituteSubmitterId,
    ) -> Result<Option<SubstituteSubmitter>, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data
            .substitute_submitters
            .get(&substitute_submitter_id)
            .cloned())
    }

    pub fn get_last_event_id(&self) -> Result<usize, AppError> {
        let data = self
            .data
            .read()
            .map_err(|_| AppError::InternalServerError)?;

        Ok(data.last_event_id)
    }
}

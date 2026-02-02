use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    candidate_lists::{CandidateList, CandidateListId},
    persons::{Person, PersonId},
    political_groups::{
        AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub enum AppEvent {
    UpdatePoliticalGroup(PoliticalGroup),

    CreatePerson(Person),
    UpdatePerson(Person),
    DeletePerson(PersonId),

    CreateCandidateList(CandidateList),
    UpdateCandidateList(CandidateList),
    DeleteCandidateList(CandidateListId),

    CreateAuthorisedAgent(AuthorisedAgent),
    UpdateAuthorisedAgent(AuthorisedAgent),
    DeleteAuthorisedAgent(AuthorisedAgentId),

    CreateListSubmitter(ListSubmitter),
    UpdateListSubmitter(ListSubmitter),
    DeleteListSubmitter(ListSubmitterId),
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppStoreData {
    political_group: PoliticalGroup,
    persons: HashMap<PersonId, Person>,
    candidate_lists: HashMap<CandidateListId, CandidateList>,
    authorised_agents: HashMap<AuthorisedAgentId, AuthorisedAgent>,
    list_submitters: HashMap<ListSubmitterId, ListSubmitter>,
    events: Vec<AppEvent>,
}

#[derive(Default, Clone)]
pub struct AppStore {
    data: Arc<RwLock<AppStoreData>>,
}

impl AppStore {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_candidate_lists(&self) -> Vec<CandidateList> {
        let data = self.data.read().unwrap();

        let mut lists: Vec<CandidateList> = data.candidate_lists.values().cloned().collect();
        lists.sort_by(|a, b| a.created_at.cmp(&b.created_at).then_with(|| a.id.cmp(&b.id)));
        lists
    }

    pub fn get_political_group(&self) -> PoliticalGroup {
        let data = self.data.read().unwrap();

        data.political_group.clone()
    }

    pub fn get_persons(&self) -> Vec<Person> {
        let data = self.data.read().unwrap();

        data.persons.values().cloned().collect()
    }

    pub fn get_authorised_agents(&self) -> Vec<AuthorisedAgent> {
        let data = self.data.read().unwrap();

        data.authorised_agents.values().cloned().collect()
    }

    pub fn get_list_submitters(&self) -> Vec<ListSubmitter> {
        let data = self.data.read().unwrap();

        data.list_submitters.values().cloned().collect()
    }

    pub fn get_person_count(&self) -> usize {
        let data = self.data.read().unwrap();

        data.persons.len()
    }

    pub fn get_candidate_list(&self, list_id: CandidateListId) -> Result<CandidateList, AppError> {
        let data = self.data.read().unwrap();

        data.candidate_lists
            .get(&list_id)
            .cloned()
            .ok_or_else(|| AppError::GenericNotFound)
    }

    pub fn get_person(&self, person_id: PersonId) -> Result<Person, AppError> {
        let data = self.data.read().unwrap();

        data.persons
            .get(&person_id)
            .cloned()
            .ok_or_else(|| AppError::GenericNotFound)
    }

    pub fn get_authorised_agent(
        &self,
        authorised_agent_id: AuthorisedAgentId,
    ) -> Result<AuthorisedAgent, AppError> {
        let data = self.data.read().unwrap();

        data.authorised_agents
            .get(&authorised_agent_id)
            .cloned()
            .ok_or_else(|| AppError::GenericNotFound)
    }

    pub fn get_list_submitter(
        &self,
        list_submitter_id: ListSubmitterId,
    ) -> Result<ListSubmitter, AppError> {
        let data = self.data.read().unwrap();

        data.list_submitters
            .get(&list_submitter_id)
            .cloned()
            .ok_or_else(|| AppError::GenericNotFound)
    }

    pub async fn update(&self, event: AppEvent) -> Result<(), AppError> {
        let mut data = self.data.write().unwrap();

        match &event {
            AppEvent::UpdatePoliticalGroup(pg) => {
                data.political_group = pg.clone();
            }
            AppEvent::CreatePerson(person) => {
                data.persons.insert(person.id, person.clone());
            }
            AppEvent::UpdatePerson(person) => {
                if let Some(existing) = data.persons.get_mut(&person.id) {
                    *existing = person.clone();
                }
            }
            AppEvent::DeletePerson(person_id) => {
                data.persons.remove(person_id);
            }
            AppEvent::CreateCandidateList(cl) => {
                data.candidate_lists.insert(cl.id, cl.clone());
            }
            AppEvent::UpdateCandidateList(cl) => {
                if let Some(existing) = data.candidate_lists.get_mut(&cl.id) {
                    *existing = cl.clone();
                }
            }
            AppEvent::DeleteCandidateList(cl_id) => {
                data.candidate_lists.remove(cl_id);
            }
            AppEvent::CreateAuthorisedAgent(aa) => {
                data.authorised_agents.insert(aa.id, aa.clone());
            }
            AppEvent::UpdateAuthorisedAgent(aa) => {
                if let Some(existing) = data.authorised_agents.get_mut(&aa.id) {
                    *existing = aa.clone();
                }
            }
            AppEvent::DeleteAuthorisedAgent(aa_id) => {
                data.authorised_agents.remove(aa_id);
            }
            AppEvent::CreateListSubmitter(ls) => {
                data.list_submitters.insert(ls.id, ls.clone());
            }
            AppEvent::UpdateListSubmitter(ls) => {
                if let Some(existing) = data.list_submitters.get_mut(&ls.id) {
                    *existing = ls.clone();
                }
            }
            AppEvent::DeleteListSubmitter(ls_id) => {
                data.list_submitters.remove(ls_id);
            }
        }

        data.events.push(event);

        Ok(())
    }
}

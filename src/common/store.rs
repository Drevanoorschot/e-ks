use std::sync::{Arc, RwLock};

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
    persons: Vec<Person>,
    candidate_lists: Vec<CandidateList>,
    authororized_agents: Vec<AuthorisedAgent>,
    list_submitters: Vec<ListSubmitter>,
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

        data.candidate_lists.clone()
    }

    pub fn get_political_group(&self) -> PoliticalGroup {
        let data = self.data.read().unwrap();

        data.political_group.clone()
    }

    pub fn get_persons(&self) -> Vec<Person> {
        let data = self.data.read().unwrap();

        data.persons.clone()
    }

    pub fn get_authorised_agents(&self) -> Vec<AuthorisedAgent> {
        let data = self.data.read().unwrap();

        data.authororized_agents.clone()
    }

    pub fn get_list_submitters(&self) -> Vec<ListSubmitter> {
        let data = self.data.read().unwrap();

        data.list_submitters.clone()
    }

    pub fn get_person_count(&self) -> usize {
        let data = self.data.read().unwrap();

        data.persons.len()
    }

    pub async fn update(&self, event: AppEvent) -> Result<(), AppError> {
        let mut data = self.data.write().unwrap();

        match &event {
            AppEvent::UpdatePoliticalGroup(pg) => {
                data.political_group = pg.clone();
            }
            AppEvent::CreatePerson(person) => {
                data.persons.push(person.clone());
            }
            AppEvent::UpdatePerson(person) => {
                if let Some(existing) = data.persons.iter_mut().find(|p| p.id == person.id) {
                    *existing = person.clone();
                }
            }
            AppEvent::DeletePerson(person_id) => {
                data.persons.retain(|p| &p.id != person_id);
            }
            AppEvent::CreateCandidateList(cl) => {
                data.candidate_lists.push(cl.clone());
            }
            AppEvent::UpdateCandidateList(cl) => {
                if let Some(existing) = data.candidate_lists.iter_mut().find(|c| c.id == cl.id) {
                    *existing = cl.clone();
                }
            }
            AppEvent::DeleteCandidateList(cl_id) => {
                data.candidate_lists.retain(|c| &c.id != cl_id);
            }
            AppEvent::CreateAuthorisedAgent(aa) => {
                data.authororized_agents.push(aa.clone());
            }
            AppEvent::UpdateAuthorisedAgent(aa) => {
                if let Some(existing) = data.authororized_agents.iter_mut().find(|a| a.id == aa.id)
                {
                    *existing = aa.clone();
                }
            }
            AppEvent::DeleteAuthorisedAgent(aa_id) => {
                data.authororized_agents.retain(|a| &a.id != aa_id);
            }
            AppEvent::CreateListSubmitter(ls) => {
                data.list_submitters.push(ls.clone());
            }
            AppEvent::UpdateListSubmitter(ls) => {
                if let Some(existing) = data.list_submitters.iter_mut().find(|l| l.id == ls.id) {
                    *existing = ls.clone();
                }
            }
            AppEvent::DeleteListSubmitter(ls_id) => {
                data.list_submitters.retain(|l| &l.id != ls_id);
            }
        }

        data.events.push(event);

        Ok(())
    }
}

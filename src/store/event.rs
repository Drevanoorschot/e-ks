use serde::{Deserialize, Serialize};

use crate::{
    candidate_lists::{CandidateList, CandidateListId},
    persons::{Person, PersonId},
    political_groups::{
        AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup,
        SubstituteSubmitter, SubstituteSubmitterId,
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

    CreateSubstituteSubmitter(SubstituteSubmitter),
    UpdateSubstituteSubmitter(SubstituteSubmitter),
    DeleteSubstituteSubmitter(SubstituteSubmitterId),
}

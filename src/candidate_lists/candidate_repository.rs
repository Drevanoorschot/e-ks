use chrono::Utc;
use std::collections::BTreeMap;

use crate::{
    AppError, AppStore,
    candidate_lists::{self, Candidate, CandidateList, CandidateListId, FullCandidateList},
    common::store::AppEvent,
    persons::{Person, PersonId},
};

pub async fn get_full_candidate_list(
    store: &AppStore,
    list_id: CandidateListId,
) -> Result<Option<FullCandidateList>, AppError> {
    let list = candidate_lists::get_candidate_list(store, list_id)?;
    let Some(list) = list else {
        return Ok(None);
    };

    let full_list = build_full_candidate_list(store, list)?;

    Ok(Some(full_list))
}

pub async fn update_candidate_list_order(
    store: &AppStore,
    list_id: CandidateListId,
    person_ids: &[PersonId],
) -> Result<FullCandidateList, AppError> {
    let Some(mut list) = candidate_lists::get_candidate_list(store, list_id)? else {
        return Err(AppError::NotFound("candidate list not found".to_string()));
    };

    ensure_persons_exist(store, person_ids)?;

    list.candidates = person_ids.to_vec();
    list.updated_at = Utc::now();

    store
        .update(AppEvent::UpdateCandidateList(list.clone()))
        .await?;

    build_full_candidate_list(store, list)
}

pub async fn append_candidate_to_list(
    store: &AppStore,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<(), AppError> {
    let Some(mut list) = candidate_lists::get_candidate_list(store, list_id)? else {
        return Err(AppError::NotFound("candidate list not found".to_string()));
    };

    ensure_persons_exist(store, &[person_id])?;

    if !list.candidates.contains(&person_id) {
        list.candidates.push(person_id);
        list.updated_at = Utc::now();
        store.update(AppEvent::UpdateCandidateList(list)).await?;
    }

    Ok(())
}

pub async fn remove_candidate(store: &AppStore, person_id: PersonId) -> Result<(), AppError> {
    let mut removed_any = false;
    let lists = store.get_candidate_lists();

    for mut list in lists {
        if list.candidates.contains(&person_id) {
            list.candidates.retain(|id| *id != person_id);
            list.updated_at = Utc::now();
            store.update(AppEvent::UpdateCandidateList(list)).await?;
            removed_any = true;
        }
    }

    if !removed_any {
        return Err(AppError::NotFound(
            "candidate not found in any list".to_string(),
        ));
    }

    Ok(())
}

pub async fn remove_candidate_from_list(
    store: &AppStore,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<(), AppError> {
    let Some(mut list) = candidate_lists::get_candidate_list(store, list_id)? else {
        return Err(AppError::NotFound("candidate list not found".to_string()));
    };

    if !list.candidates.contains(&person_id) {
        return Err(AppError::NotFound(
            "candidate not found on list".to_string(),
        ));
    }

    list.candidates.retain(|id| *id != person_id);
    list.updated_at = Utc::now();
    store.update(AppEvent::UpdateCandidateList(list)).await?;

    Ok(())
}

pub async fn get_candidate(
    store: &AppStore,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<Candidate, AppError> {
    let Some(list) = candidate_lists::get_candidate_list(store, list_id)? else {
        return Err(AppError::NotFound("candidate list not found".to_string()));
    };

    let position = list
        .candidates
        .iter()
        .position(|id| *id == person_id)
        .map(|index| index + 1)
        .ok_or_else(|| AppError::NotFound("candidate not found on list".to_string()))?;

    let person = store
        .get_persons()
        .into_iter()
        .find(|p| p.id == person_id)
        .ok_or_else(|| AppError::NotFound("person not found".to_string()))?;

    Ok(Candidate {
        list_id,
        position,
        person,
    })
}

pub fn list_persons_not_on_candidate_list(
    store: &AppStore,
    list_id: CandidateListId,
) -> Result<Vec<Person>, AppError> {
    let list = candidate_lists::get_candidate_list(store, list_id)?;
    let Some(list) = list else {
        return Err(AppError::NotFound("candidate list not found".to_string()));
    };

    let existing: BTreeMap<PersonId, ()> = list.candidates.into_iter().map(|id| (id, ())).collect();

    Ok(store
        .get_persons()
        .into_iter()
        .filter(|person| !existing.contains_key(&person.id))
        .collect())
}

fn build_full_candidate_list(
    store: &AppStore,
    list: CandidateList,
) -> Result<FullCandidateList, AppError> {
    let persons: BTreeMap<PersonId, Person> = store
        .get_persons()
        .into_iter()
        .map(|person| (person.id, person))
        .collect();

    let candidates = list
        .candidates
        .iter()
        .enumerate()
        .map(|(index, person_id)| {
            let person = persons
                .get(person_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound("person not found".to_string()))?;
            Ok(Candidate {
                list_id: list.id,
                position: index + 1,
                person,
            })
        })
        .collect::<Result<Vec<Candidate>, AppError>>()?;

    Ok(FullCandidateList { list, candidates })
}

fn ensure_persons_exist(store: &AppStore, person_ids: &[PersonId]) -> Result<(), AppError> {
    let existing: BTreeMap<PersonId, ()> = store
        .get_persons()
        .into_iter()
        .map(|person| (person.id, ()))
        .collect();

    if person_ids.iter().any(|id| !existing.contains_key(id)) {
        return Err(AppError::NotFound("person not found".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        AppStore, candidate_lists,
        common::store::AppEvent,
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    #[tokio::test]
    async fn get_candidate_list_includes_candidates() -> Result<(), AppError> {
        let store = AppStore::default();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&store, &list).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        update_candidate_list_order(&store, list_id, &[person_a.id, person_b.id]).await?;

        let detail = get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(2, detail.candidates.len());
        assert_eq!(person_a.id, detail.candidates[0].person.id);
        assert_eq!(person_b.id, detail.candidates[1].person.id);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_order_returns_not_found() -> Result<(), AppError> {
        let store = AppStore::default();
        let err = update_candidate_list_order(&store, CandidateListId::new(), &[])
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));

        Ok(())
    }

    #[tokio::test]
    async fn get_full_candidate_list_returns_none_for_missing_list() -> Result<(), AppError> {
        let store = AppStore::default();
        let missing = get_full_candidate_list(&store, CandidateListId::new()).await?;
        assert!(missing.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_append_candidate_to_list() -> Result<(), AppError> {
        let store = AppStore::default();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&store, &list).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;

        append_candidate_to_list(&store, list_id, person_a.id).await?;
        append_candidate_to_list(&store, list_id, person_b.id).await?;

        let detail = get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");

        assert_eq!(detail.candidates.len(), 2);
        assert_eq!(detail.candidates[0].person.id, person_a.id);
        assert_eq!(detail.candidates[0].position, 1);
        assert_eq!(detail.candidates[1].person.id, person_b.id);
        assert_eq!(detail.candidates[1].position, 2);

        Ok(())
    }

    #[tokio::test]
    async fn append_candidate_to_list_returns_not_found() -> Result<(), AppError> {
        let store = AppStore::default();
        let err = append_candidate_to_list(&store, CandidateListId::new(), PersonId::new())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));

        Ok(())
    }

    #[tokio::test]
    async fn remove_candidate_removes_from_list() -> Result<(), AppError> {
        let store = AppStore::default();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&store, &list).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        append_candidate_to_list(&store, list_id, person_a.id).await?;
        append_candidate_to_list(&store, list_id, person_b.id).await?;

        remove_candidate(&store, person_a.id).await?;

        let detail = get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(detail.candidates.len(), 1);
        assert_eq!(detail.candidates[0].person.id, person_b.id);

        Ok(())
    }

    #[tokio::test]
    async fn remove_candidate_returns_not_found() -> Result<(), AppError> {
        let store = AppStore::default();
        let err = remove_candidate(&store, PersonId::new()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_returns_candidate() -> Result<(), AppError> {
        let store = AppStore::default();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        candidate_lists::create_candidate_list(&store, &list).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        append_candidate_to_list(&store, list_id, person.id).await?;

        let candidate = get_candidate(&store, list_id, person.id).await?;
        assert_eq!(candidate.list_id, list_id);
        assert_eq!(candidate.position, 1);
        assert_eq!(candidate.person.id, person.id);

        Ok(())
    }
}

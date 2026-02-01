use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;

use crate::{
    AppError, AppStore, ElectoralDistrict,
    candidate_lists::{CandidateList, CandidateListId, CandidateListSummary},
    common::store::AppEvent,
};

pub fn list_candidate_list_summary(
    store: &AppStore,
) -> Result<Vec<CandidateListSummary>, AppError> {
    let lists = store.get_candidate_lists();

    let mut district_count = BTreeMap::<ElectoralDistrict, usize>::new();
    for list in &lists {
        for district in &list.electoral_districts {
            district_count
                .entry(*district)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }

    let summaries = lists
        .into_iter()
        .map(|list| {
            let person_count = list.candidates.len();
            let duplicate_districts = list
                .electoral_districts
                .iter()
                .filter(|district| *district_count.entry(**district).or_default() > 1)
                .cloned()
                .collect();

            CandidateListSummary {
                list,
                person_count,
                duplicate_districts,
            }
        })
        .collect();

    Ok(summaries)
}

pub fn get_candidate_list(
    store: &AppStore,
    list_id: CandidateListId,
) -> Result<Option<CandidateList>, AppError> {
    let lists = store.get_candidate_lists();
    Ok(lists.into_iter().find(|list| list.id == list_id))
}

/// Retrieves a vector of all the electoral districts that have been used in one or more candidate lists.
/// Optionally, include list ids to exclude in the aggregation
pub fn get_used_districts(
    store: &AppStore,
    exclude_list_ids: Vec<CandidateListId>,
) -> Result<Vec<ElectoralDistrict>, AppError> {
    let exclude: BTreeSet<CandidateListId> = exclude_list_ids.into_iter().collect();
    let mut used = BTreeSet::<ElectoralDistrict>::new();

    for list in store.get_candidate_lists() {
        if exclude.contains(&list.id) {
            continue;
        }

        for district in list.electoral_districts {
            used.insert(district);
        }
    }

    Ok(used.into_iter().collect())
}

pub async fn create_candidate_list(
    store: &AppStore,
    candidate_list: &CandidateList,
) -> Result<CandidateList, AppError> {
    let now = Utc::now();
    let list = CandidateList {
        created_at: now,
        updated_at: now,
        ..candidate_list.clone()
    };

    store
        .update(AppEvent::CreateCandidateList(list.clone()))
        .await?;

    Ok(list)
}

pub async fn update_candidate_list(
    store: &AppStore,
    updated_candidate_list: &CandidateList,
) -> Result<CandidateList, AppError> {
    let Some(existing) = store
        .get_candidate_lists()
        .into_iter()
        .find(|list| list.id == updated_candidate_list.id)
    else {
        return Err(AppError::NotFound("candidate list not found".to_string()));
    };

    let updated = CandidateList {
        electoral_districts: updated_candidate_list.electoral_districts.clone(),
        candidates: existing.candidates,
        created_at: existing.created_at,
        updated_at: Utc::now(),
        ..existing
    };

    store
        .update(AppEvent::UpdateCandidateList(updated.clone()))
        .await?;

    Ok(updated)
}

pub async fn remove_candidate_list(
    store: &AppStore,
    list_id: CandidateListId,
) -> Result<(), AppError> {
    store.update(AppEvent::DeleteCandidateList(list_id)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use chrono::Utc;

    use crate::{
        AppStore, candidate_lists,
        common::store::AppEvent,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    async fn insert_list(
        store: &AppStore,
        electoral_districts: Vec<ElectoralDistrict>,
    ) -> Result<CandidateList, AppError> {
        let list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts,
            candidates: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        create_candidate_list(store, &list).await
    }

    #[tokio::test]
    async fn create_and_list_candidate_lists() -> Result<(), AppError> {
        let store = AppStore::default();
        let list = sample_candidate_list(CandidateListId::new());

        create_candidate_list(&store, &list).await?;

        let lists = list_candidate_list_summary(&store)?;
        assert_eq!(1, lists.len());
        assert_eq!(list.id, lists[0].list.id);
        assert_eq!(0, lists[0].person_count);
        assert_eq!(0, lists[0].duplicate_districts.len());

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_list_summaries_with_duplicate_districts() -> Result<(), AppError> {
        let store = AppStore::default();
        // setup
        let list1 = insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        let list2 = insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::GR]).await?;

        let list3 = insert_list(&store, vec![ElectoralDistrict::OV, ElectoralDistrict::GR]).await?;

        // test
        let lists = list_candidate_list_summary(&store)?;

        // verification
        assert_eq!(3, lists.len());

        let list_summary1 = lists.iter().find(|list| list.list.id == list1.id).unwrap();
        let list_summary2 = lists.iter().find(|list| list.list.id == list2.id).unwrap();
        let list_summary3 = lists.iter().find(|list| list.list.id == list3.id).unwrap();

        // list 1 clashes on UT with list 2
        assert_eq!(
            vec![ElectoralDistrict::UT],
            list_summary1.duplicate_districts
        );

        // list 2 clashes on UT with list 1 and on GR with list 3
        assert_eq!(2, list_summary2.duplicate_districts.len());
        assert!(
            list_summary2
                .duplicate_districts
                .contains(&ElectoralDistrict::UT)
        );
        assert!(
            list_summary2
                .duplicate_districts
                .contains(&ElectoralDistrict::GR)
        );

        // list 3 clashes on GR with list 2
        assert_eq!(
            vec![ElectoralDistrict::GR],
            list_summary3.duplicate_districts
        );

        Ok(())
    }

    #[tokio::test]
    async fn list_candidate_list_orders_by_created_at() -> Result<(), AppError> {
        let store = AppStore::default();
        let list_early = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            candidates: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let list_late = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::OV],
            candidates: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        create_candidate_list(&store, &list_early).await?;
        create_candidate_list(&store, &list_late).await?;

        let lists = store.get_candidate_lists();
        assert_eq!(lists.len(), 2);
        assert_eq!(lists[0].id, list_early.id);
        assert_eq!(lists[1].id, list_late.id);

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_list_returns_list() -> Result<(), AppError> {
        let store = AppStore::default();
        let list = sample_candidate_list(CandidateListId::new());

        create_candidate_list(&store, &list).await?;

        let loaded = get_candidate_list(&store, list.id)?.expect("candidate list");

        assert_eq!(loaded.id, list.id);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_updates_districts() -> Result<(), AppError> {
        let store = AppStore::default();
        let list = sample_candidate_list(CandidateListId::new());

        create_candidate_list(&store, &list).await?;

        let updated = update_candidate_list(
            &store,
            &CandidateList {
                electoral_districts: vec![ElectoralDistrict::DR, ElectoralDistrict::OV],
                ..list.clone()
            },
        )
        .await?;

        assert_eq!(updated.id, list.id);
        assert_eq!(
            updated.electoral_districts,
            vec![ElectoralDistrict::DR, ElectoralDistrict::OV]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_used_districts() -> Result<(), AppError> {
        let store = AppStore::default();
        // setup
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::OV]).await?;
        insert_list(&store, vec![]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            get_used_districts(&store, vec![])?.into_iter().collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn get_used_districts_no_lists() -> Result<(), AppError> {
        let store = AppStore::default();
        let result = get_used_districts(&store, vec![])?;

        assert_eq!(Vec::<ElectoralDistrict>::new(), result);

        Ok(())
    }

    #[tokio::test]
    async fn get_used_districts_double_districts() -> Result<(), AppError> {
        let store = AppStore::default();
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::OV]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            get_used_districts(&store, vec![])?.into_iter().collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn get_used_district_with_exclude() -> Result<(), AppError> {
        let store = AppStore::default();
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::GR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::GR, ElectoralDistrict::OV]).await?;

        let exclude_id = insert_list(&store, vec![ElectoralDistrict::GR, ElectoralDistrict::LI])
            .await?
            .id;

        // test
        let result: BTreeSet<ElectoralDistrict> = get_used_districts(&store, vec![exclude_id])?
            .into_iter()
            .collect();

        // verify
        assert_eq!(expected, result);

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_candidate_list() -> Result<(), AppError> {
        let store = AppStore::default();
        // setup
        let list_a = sample_candidate_list(CandidateListId::new());
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let list_b = sample_candidate_list(CandidateListId::new());
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        create_candidate_list(&store, &list_a).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        candidate_lists::update_candidate_list_order(&store, list_a.id, &[person_a.id]).await?;

        create_candidate_list(&store, &list_b).await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        candidate_lists::update_candidate_list_order(&store, list_b.id, &[person_b.id]).await?;

        // test
        remove_candidate_list(&store, list_a.id).await?;

        // verify
        let lists = list_candidate_list_summary(&store)?;
        let list_b_from_db = candidate_lists::get_full_candidate_list(&store, list_b.id)
            .await?
            .unwrap();
        // one list remains
        assert_eq!(1, lists.len());
        // the correct list got deleted
        assert_eq!(list_b.id, lists[0].list.id);
        // and only persons got removed associated with the deleted list
        assert_eq!(1, lists[0].person_count);
        assert_eq!(person_b.id, list_b_from_db.candidates[0].person.id);
        // no duplicate districts
        assert_eq!(0, lists[0].duplicate_districts.len());

        Ok(())
    }
}

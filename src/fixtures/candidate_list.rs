use chrono::Utc;
use uuid::Uuid;

use crate::{
    AppError, AppStore, ElectionConfig,
    candidate_lists::{self, CandidateList},
    pagination::SortDirection,
    persons::{self, Person, PersonId},
};

const FIXTURE_CANDIDATE_LIST_SIZE: usize = 55;

fn collect_person_ids(persons: Vec<Person>) -> Vec<PersonId> {
    persons.into_iter().map(|person| person.id).collect()
}

pub async fn load(store: &AppStore) -> Result<(), AppError> {
    let electoral_districts = ElectionConfig::EK2027.electoral_districts().to_vec();
    let persons = persons::list_persons(
        store,
        FIXTURE_CANDIDATE_LIST_SIZE as i64,
        0,
        &persons::PersonSort::CreatedAt,
        &SortDirection::Asc,
    );
    let person_ids = collect_person_ids(persons);
    let uuid = Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        b"the_one_and_only_fixture_candidate_list",
    );

    let candidate_list = CandidateList {
        id: uuid.into(),
        electoral_districts,
        candidates: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let candidate_list = candidate_lists::create_candidate_list(store, &candidate_list).await?;

    // Persist the ordered set of persons to ensure deterministic candidate positions.
    candidate_lists::update_candidate_list_order(store, candidate_list.id, &person_ids).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load() {
        let store = AppStore::default();
        crate::fixtures::persons::load(&store).await.unwrap();
        load(&store).await.unwrap();

        let lists = candidate_lists::list_candidate_list_summary(&store).unwrap();

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].person_count, FIXTURE_CANDIDATE_LIST_SIZE);
    }
}

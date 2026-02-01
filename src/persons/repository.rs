use std::cmp::Ordering;

use chrono::Utc;

use crate::{
    AppError, AppStore,
    common::store::AppEvent,
    pagination::SortDirection,
    persons::{Gender, Person, PersonId, PersonSort},
};

pub fn count_persons(store: &AppStore) -> usize {
    store.get_person_count()
}

pub fn list_persons(
    store: &AppStore,
    limit: i64,
    offset: i64,
    sort_field: &PersonSort,
    sort_direction: &SortDirection,
) -> Vec<Person> {
    let mut persons = store.get_persons();
    persons.sort_by(|a, b| compare_persons(a, b, sort_field));

    if matches!(sort_direction, SortDirection::Desc) {
        persons.reverse();
    }

    let offset = offset.max(0) as usize;
    let limit = limit.max(0) as usize;

    persons.into_iter().skip(offset).take(limit).collect()
}

pub fn get_person(store: &AppStore, person_id: PersonId) -> Option<Person> {
    store
        .get_persons()
        .into_iter()
        .find(|person| person.id == person_id)
}

pub async fn create_person(store: &AppStore, new_person: &Person) -> Result<Person, AppError> {
    let now = Utc::now();
    let person = Person {
        created_at: now,
        updated_at: now,
        ..new_person.clone()
    };

    store.update(AppEvent::CreatePerson(person.clone())).await?;

    Ok(person)
}

pub async fn update_person(store: &AppStore, updated_person: &Person) -> Result<Person, AppError> {
    let existing = get_person(store, updated_person.id)
        .ok_or_else(|| AppError::NotFound("person not found".to_string()))?;

    let updated = Person {
        locality: existing.locality,
        postal_code: existing.postal_code,
        house_number: existing.house_number,
        house_number_addition: existing.house_number_addition,
        street_name: existing.street_name,
        created_at: existing.created_at,
        updated_at: Utc::now(),
        ..updated_person.clone()
    };

    store
        .update(AppEvent::UpdatePerson(updated.clone()))
        .await?;

    Ok(updated)
}

pub async fn update_address(store: &AppStore, updated_person: &Person) -> Result<Person, AppError> {
    let existing = get_person(store, updated_person.id)
        .ok_or_else(|| AppError::NotFound("person not found".to_string()))?;

    let updated = Person {
        locality: updated_person.locality.clone(),
        postal_code: updated_person.postal_code.clone(),
        house_number: updated_person.house_number.clone(),
        house_number_addition: updated_person.house_number_addition.clone(),
        street_name: updated_person.street_name.clone(),
        updated_at: Utc::now(),
        ..existing
    };

    store
        .update(AppEvent::UpdatePerson(updated.clone()))
        .await?;

    Ok(updated)
}

pub async fn remove_person(store: &AppStore, person_id: PersonId) -> Result<(), AppError> {
    store.update(AppEvent::DeletePerson(person_id)).await?;

    Ok(())
}

fn compare_persons(a: &Person, b: &Person, sort_field: &PersonSort) -> Ordering {
    match sort_field {
        PersonSort::LastName => cmp_string(&a.last_name, &b.last_name)
            .then_with(|| cmp_option_string(&a.last_name_prefix, &b.last_name_prefix))
            .then_with(|| cmp_string(&a.initials, &b.initials))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::FirstName => cmp_option_string(&a.first_name, &b.first_name)
            .then_with(|| cmp_string(&a.last_name, &b.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::Initials => cmp_string(&a.initials, &b.initials)
            .then_with(|| cmp_string(&a.last_name, &b.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::Gender => cmp_gender(&a.gender, &b.gender)
            .then_with(|| cmp_string(&a.last_name, &b.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::PlaceOfResidence => {
            cmp_option_string(&a.place_of_residence, &b.place_of_residence)
                .then_with(|| cmp_string(&a.last_name, &b.last_name))
                .then_with(|| a.id.cmp(&b.id))
        }
        PersonSort::CreatedAt => a
            .created_at
            .cmp(&b.created_at)
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::UpdatedAt => a
            .updated_at
            .cmp(&b.updated_at)
            .then_with(|| a.id.cmp(&b.id)),
    }
}

fn cmp_string(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}

fn cmp_option_string(a: &Option<String>, b: &Option<String>) -> Ordering {
    cmp_string(a.as_deref().unwrap_or(""), b.as_deref().unwrap_or(""))
}

fn cmp_gender(a: &Option<Gender>, b: &Option<Gender>) -> Ordering {
    gender_rank(a).cmp(&gender_rank(b))
}

fn gender_rank(gender: &Option<Gender>) -> u8 {
    match gender {
        None => 0,
        Some(Gender::Female) => 1,
        Some(Gender::Male) => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        AppStore,
        pagination::SortDirection,
        persons::PersonSort,
        test_utils::{sample_person, sample_person_with_last_name},
    };

    #[tokio::test]
    async fn create_and_get_person() -> Result<(), AppError> {
        let store = AppStore::default();
        let id = PersonId::new();
        let person = sample_person(id);

        create_person(&store, &person).await?;

        let loaded = get_person(&store, id).expect("person");
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.last_name, "Jansen");

        Ok(())
    }

    #[tokio::test]
    async fn list_and_count_persons() -> Result<(), AppError> {
        let store = AppStore::default();
        create_person(
            &store,
            &sample_person_with_last_name(PersonId::new(), "Jansen"),
        )
        .await?;
        create_person(
            &store,
            &sample_person_with_last_name(PersonId::new(), "Bakker"),
        )
        .await?;

        let total = count_persons(&store);
        assert_eq!(total, 2);

        let persons = list_persons(&store, 10, 0, &PersonSort::LastName, &SortDirection::Asc);
        assert_eq!(persons.len(), 2);
        assert_eq!(persons[0].last_name, "Bakker");

        Ok(())
    }

    #[tokio::test]
    async fn update_person_overwrites_fields() -> Result<(), AppError> {
        let store = AppStore::default();
        let id = PersonId::new();
        let mut person = sample_person(id);

        create_person(&store, &person).await?;

        person.last_name = "Updated".to_string();
        update_person(&store, &person).await?;

        let updated = get_person(&store, id).expect("person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn remove_person_deletes_record() -> Result<(), AppError> {
        let store = AppStore::default();
        let id = PersonId::new();
        let person = sample_person(id);

        create_person(&store, &person).await?;
        remove_person(&store, id).await?;

        let missing = get_person(&store, id);
        assert!(missing.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn update_address_overwrites_fields() -> Result<(), AppError> {
        let store = AppStore::default();
        let id = PersonId::new();
        let mut person = sample_person(id);

        create_person(&store, &person).await?;

        person.locality = Some("Nieuwegein".to_string());
        person.postal_code = Some("9999 ZZ".to_string());
        person.house_number = Some("99".to_string());
        person.house_number_addition = None;
        person.street_name = Some("Nieuweweg".to_string());

        update_address(&store, &person).await?;

        let updated = get_person(&store, id).expect("person");
        assert_eq!(updated.locality, Some("Nieuwegein".to_string()));
        assert_eq!(updated.postal_code, Some("9999 ZZ".to_string()));
        assert_eq!(updated.house_number, Some("99".to_string()));
        assert_eq!(updated.house_number_addition, None);
        assert_eq!(updated.street_name, Some("Nieuweweg".to_string()));

        Ok(())
    }
}

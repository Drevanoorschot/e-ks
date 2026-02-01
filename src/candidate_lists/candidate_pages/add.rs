use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use serde::Deserialize;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    candidate_lists::{self, CandidateList, FullCandidateList, pages::AddCandidatePath},
    filters,
    persons::{Person, PersonId},
};

#[derive(Template)]
#[template(path = "candidates/add_existing.html")]
struct AddExistingPersonTemplate {
    full_list: FullCandidateList,
    persons: Vec<Person>,
}

pub async fn add_existing_person(
    _: AddCandidatePath,
    context: Context,
    full_list: FullCandidateList,
    State(store): State<AppStore>,
) -> Result<impl IntoResponse, AppError> {
    let persons = candidate_lists::list_persons_not_on_candidate_list(&store, full_list.id())?;

    Ok(HtmlTemplate(
        AddExistingPersonTemplate { full_list, persons },
        context,
    ))
}

#[derive(Deserialize)]
pub struct AddPersonForm {
    pub person_id: PersonId,
}

pub async fn add_person_to_candidate_list(
    _: AddCandidatePath,
    full_list: FullCandidateList,
    State(store): State<AppStore>,
    Form(form): Form<AddPersonForm>,
) -> Result<Response, AppError> {
    let redirect = Redirect::to(&full_list.list.view_path()).into_response();
    let person_exists = store
        .get_persons()
        .iter()
        .any(|person| person.id == form.person_id);

    if full_list.contains(form.person_id) || !person_exists {
        return Ok(redirect);
    }

    candidate_lists::append_candidate_to_list(&store, full_list.id(), form.person_id).await?;

    Ok(redirect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;

    use crate::{
        AppStore, Context,
        candidate_lists::{self, CandidateListId},
        common::store::AppEvent,
        persons::PersonId,
        test_utils::{
            self, response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };

    #[tokio::test]
    async fn view_candidate_list_renders_persons() -> Result<(), AppError> {
        let store = AppStore::default();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        candidate_lists::create_candidate_list(&store, &list).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;

        let full_list = candidate_lists::get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");

        let response = add_existing_person(
            AddCandidatePath { list_id },
            Context::new_test_without_db(),
            full_list,
            State(store),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.add_candidate_path()));
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_adds_and_redirects() -> Result<(), AppError> {
        let store = AppStore::default();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&store, &list).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;

        let full_list = candidate_lists::get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            State(store.clone()),
            Form(AddPersonForm {
                person_id: person.id,
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, list.view_path());

        let full_list = candidate_lists::get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, person.id);

        Ok(())
    }

    #[tokio::test]
    async fn add_person_to_candidate_list_redirects_when_person_not_on_list() -> Result<(), AppError>
    {
        let store = AppStore::default();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let existing_person = sample_person_with_last_name(PersonId::new(), "Jansen");
        let new_person = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&store, &list).await?;
        store
            .update(AppEvent::CreatePerson(existing_person.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(new_person.clone()))
            .await?;
        candidate_lists::update_candidate_list_order(&store, list_id, &[existing_person.id])
            .await?;

        let full_list = candidate_lists::get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");

        let response = add_person_to_candidate_list(
            AddCandidatePath { list_id },
            full_list,
            State(store.clone()),
            Form(AddPersonForm {
                person_id: new_person.id,
            }),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, list.view_path());

        let full_list = candidate_lists::get_full_candidate_list(&store, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, existing_person.id);
        assert_eq!(full_list.candidates[1].person.id, new_person.id);

        Ok(())
    }
}

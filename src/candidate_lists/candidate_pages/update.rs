use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use chrono::Utc;

use crate::{
    AppError, AppResponse, AppStore, Context, HtmlTemplate,
    candidate_lists::{
        Candidate, CandidateList, FullCandidateList, candidate_pages::CandidateListEditPersonPath,
    },
    AppEvent,
    filters,
    form::{FormData, Validate},
    persons::{COUNTRY_CODES, PersonForm},
};

#[derive(Template)]
#[template(path = "candidates/update.html")]
struct PersonUpdateTemplate {
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<PersonForm>,
    countries: &'static [&'static str],
}

pub async fn edit_person_form(
    _: CandidateListEditPersonPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(
                PersonForm::from(candidate.person.clone()),
                &context.csrf_tokens,
            ),
            candidate,
            full_list,
            countries: &COUNTRY_CODES,
        },
        context,
    ))
}

pub async fn update_person(
    _: CandidateListEditPersonPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    State(store): State<AppStore>,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate.person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                candidate,
                full_list,
                form: form_data,
                countries: &COUNTRY_CODES,
            },
            context,
        )
        .into_response()),
        Ok(mut person) => {
            person.updated_at = Utc::now();
            store.update(AppEvent::UpdatePerson(person)).await?;

            Ok(Redirect::to(&full_list.list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;

    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        AppEvent,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person, sample_person_form,
        },
    };

    #[sqlx::test]
    async fn edit_person_form_renders_candidate(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id)
            .await?
            .expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let response = edit_person_form(
            CandidateListEditPersonPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test_without_db(),
            full_list,
            candidate,
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id)
            .await?
            .expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_person(
            CandidateListEditPersonPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(store.clone()),
            Form(form),
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

        let updated = store
            .get_persons()?
            .into_iter()
            .find(|p| p.id == person.id)
            .expect("updated person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_invalid_form_renders_template(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id)
            .await?
            .expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_person(
            CandidateListEditPersonPath {
                list_id,
                person_id: person.id,
            },
            context,
            full_list,
            candidate,
            State(store),
            Form(form),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}

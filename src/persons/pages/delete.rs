use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, candidate_lists,
    form::{EmptyForm, Validate},
    persons::{self, Person, pages::DeletePersonPath},
};

pub async fn delete_person(
    DeletePersonPath { person_id }: DeletePersonPath,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            // TODO: set error flash message
            Ok(Redirect::to(&Person::list_path()).into_response())
        }
        Ok(_) => {
            match candidate_lists::remove_candidate(&store, person_id).await {
                Err(AppError::NotFound(_)) => {
                    // Candidate was not part of any candidate list, continue deletion
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }

            persons::remove_person(&store, person_id).await?;
            // TODO: set success flash message
            Ok(Redirect::to(&Person::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::extract::Form;

    use crate::{
        AppError, AppStore, Context,
        persons::{self, PersonId},
        test_utils::sample_person,
    };

    #[tokio::test]
    async fn delete_person_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::default();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        persons::create_person(&store, &person).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_person(
            DeletePersonPath { person_id },
            context,
            State(store.clone()),
            Form(EmptyForm::new(csrf_token)),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, Person::list_path());

        let found = persons::get_person(&store, person_id);
        assert!(found.is_none());

        Ok(())
    }
}

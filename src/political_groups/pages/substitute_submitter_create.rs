use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use super::SubstituteSubmitterNewPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{
        AuthorisedAgent, ListSubmitter, PoliticalGroup, SubstituteSubmitter,
        SubstituteSubmitterForm,
    },
};

#[derive(Template)]
#[template(path = "political_groups/substitute_submitter_create.html")]
struct SubstituteSubmitterCreateTemplate {
    substitute_submitters: Vec<SubstituteSubmitter>,
    form: FormData<SubstituteSubmitterForm>,
}

pub async fn new_substitute_submitter_form(
    _: SubstituteSubmitterNewPath,
    context: Context,
    State(store): State<AppStore>,
    political_group: PoliticalGroup,
) -> Result<impl IntoResponse, AppError> {
    let substitute_submitters =
        PoliticalGroup::list_substitute_submitters(&store, political_group.id)?;

    Ok(HtmlTemplate(
        SubstituteSubmitterCreateTemplate {
            substitute_submitters,
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_substitute_submitter(
    _: SubstituteSubmitterNewPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
    Form(form): Form<SubstituteSubmitterForm>,
) -> Result<Response, AppError> {
    let substitute_submitters =
        PoliticalGroup::list_substitute_submitters(&store, political_group.id)?;

    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            SubstituteSubmitterCreateTemplate {
                substitute_submitters,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(substitute_submitter) => {
            substitute_submitter
                .create(&store, political_group.id)
                .await?;
            // TODO: set success flash message
            Ok(Redirect::to(&SubstituteSubmitter::list_path()).into_response())
        }
    }
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
        AppError, AppStore, Context,
        political_groups::{PoliticalGroupId, SubstituteSubmitter},
        test_utils::{
            response_body_string, sample_political_group, sample_substitute_submitter_form,
        },
    };

    #[sqlx::test]
    async fn new_substitute_submitter_form_renders_csrf_field(pool: sqlx::PgPool) {
        let store = AppStore::new(pool);
        let context = Context::new_test_without_db();
        let group_id = store.get_political_group().unwrap().id;

        let response = new_substitute_submitter_form(
            SubstituteSubmitterNewPath {},
            context,
            State(store),
            sample_political_group(group_id),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains(&format!("action=\"{}\"", SubstituteSubmitter::new_path())));
    }

    #[sqlx::test]
    async fn create_substitute_submitter_persists_and_redirects(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_substitute_submitter_form(&csrf_token);

        let response = create_substitute_submitter(
            SubstituteSubmitterNewPath {},
            context,
            political_group,
            State(store.clone()),
            Form(form),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, SubstituteSubmitter::list_path());

        let submitters = PoliticalGroup::list_substitute_submitters(&store, group_id)?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_substitute_submitter_invalid_form_renders_template(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_substitute_submitter(
            SubstituteSubmitterNewPath {},
            context,
            political_group,
            State(store),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}

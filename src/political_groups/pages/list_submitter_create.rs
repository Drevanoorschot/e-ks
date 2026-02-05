use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use super::ListSubmitterNewPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{
        AuthorisedAgent, ListSubmitter, ListSubmitterForm, PoliticalGroup, SubstituteSubmitter,
    },
};

#[derive(Template)]
#[template(path = "political_groups/list_submitter_create.html")]
struct ListSubmitterCreateTemplate {
    list_submitters: Vec<ListSubmitter>,
    form: FormData<ListSubmitterForm>,
}

pub async fn new_list_submitter_form(
    _: ListSubmitterNewPath,
    context: Context,
    State(store): State<AppStore>,
    political_group: PoliticalGroup,
) -> Result<impl IntoResponse, AppError> {
    let list_submitters = PoliticalGroup::list_submitters(&store, political_group.id)?;

    Ok(HtmlTemplate(
        ListSubmitterCreateTemplate {
            list_submitters,
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_list_submitter(
    _: ListSubmitterNewPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let list_submitters = PoliticalGroup::list_submitters(&store, political_group.id)?;

    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterCreateTemplate {
                list_submitters,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(list_submitter) => {
            list_submitter.create(&store, political_group.id).await?;
            // TODO: set success flash message
            Ok(Redirect::to(&ListSubmitter::list_path()).into_response())
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
    use sqlx::PgPool;

    use crate::{
        AppError, AppStore, Context,
        political_groups::{ListSubmitter, PoliticalGroupId},
        test_utils::{response_body_string, sample_list_submitter_form, sample_political_group},
    };

    #[sqlx::test]
    async fn new_list_submitter_form_renders_csrf_field(pool: PgPool) {
        let store = AppStore::new(pool);
        let context = Context::new_test_without_db();
        let group_id = store.get_political_group().unwrap().id;

        let response = new_list_submitter_form(
            ListSubmitterNewPath {},
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
        assert!(body.contains(&format!("action=\"{}\"", ListSubmitter::new_path())));
    }

    #[sqlx::test]
    async fn create_list_submitter_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_list_submitter_form(&csrf_token);

        let response = create_list_submitter(
            ListSubmitterNewPath {},
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
        assert_eq!(location, ListSubmitter::list_path());

        let submitters = PoliticalGroup::list_submitters(&store, group_id)?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_list_submitter_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_list_submitter(
            ListSubmitterNewPath {},
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

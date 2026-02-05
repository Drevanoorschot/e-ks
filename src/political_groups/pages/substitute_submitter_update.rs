use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{
        AuthorisedAgent, ListSubmitter, PoliticalGroup, SubstituteSubmitter,
        SubstituteSubmitterForm,
    },
};

use super::SubstituteSubmitterEditPath;

#[derive(Template)]
#[template(path = "political_groups/substitute_submitter_update.html")]
struct SubstituteSubmitterUpdateTemplate {
    substitute_submitters: Vec<SubstituteSubmitter>,
    substitute_submitter: SubstituteSubmitter,
    form: FormData<SubstituteSubmitterForm>,
}

pub async fn edit_substitute_submitter(
    SubstituteSubmitterEditPath { submitter_id }: SubstituteSubmitterEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
) -> Result<Response, AppError> {
    let substitute_submitter =
        PoliticalGroup::get_substitute_submitter(&store, political_group.id, submitter_id)?;
    let substitute_submitters =
        PoliticalGroup::list_substitute_submitters(&store, political_group.id)?;

    Ok(HtmlTemplate(
        SubstituteSubmitterUpdateTemplate {
            form: FormData::new_with_data(
                substitute_submitter.clone().into(),
                &context.csrf_tokens,
            ),
            substitute_submitter,
            substitute_submitters,
        },
        context,
    )
    .into_response())
}

pub async fn update_substitute_submitter(
    SubstituteSubmitterEditPath { submitter_id }: SubstituteSubmitterEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
    Form(form): Form<SubstituteSubmitterForm>,
) -> Result<Response, AppError> {
    let substitute_submitter =
        PoliticalGroup::get_substitute_submitter(&store, political_group.id, submitter_id)?;
    let substitute_submitters =
        PoliticalGroup::list_substitute_submitters(&store, political_group.id)?;

    match form.validate_update(&substitute_submitter, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            SubstituteSubmitterUpdateTemplate {
                substitute_submitter,
                form: form_data,
                substitute_submitters,
            },
            context,
        )
        .into_response()),
        Ok(substitute_submitter) => {
            substitute_submitter
                .update(&store, political_group.id)
                .await?;

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
        political_groups::{PoliticalGroupId, SubstituteSubmitter, SubstituteSubmitterId},
        test_utils::{
            response_body_string, sample_political_group, sample_substitute_submitter,
            sample_substitute_submitter_form,
        },
    };

    #[sqlx::test]
    async fn edit_substitute_submitter_renders_existing_submitter(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_group.create(&store).await?;
        substitute_submitter
            .create(&store, political_group.id)
            .await?;

        let response = edit_substitute_submitter(
            SubstituteSubmitterEditPath { submitter_id },
            Context::new_test_without_db(),
            political_group,
            State(store),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&substitute_submitter.last_name));

        Ok(())
    }

    #[sqlx::test]
    async fn update_substitute_submitter_persists_and_redirects(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_group.create(&store).await?;
        substitute_submitter
            .create(&store, political_group.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_substitute_submitter(
            SubstituteSubmitterEditPath { submitter_id },
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

        let updated = PoliticalGroup::get_substitute_submitter(&store, group_id, submitter_id)?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_substitute_submitter_invalid_form_renders_template(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_group.create(&store).await?;
        substitute_submitter
            .create(&store, political_group.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_substitute_submitter(
            SubstituteSubmitterEditPath { submitter_id },
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

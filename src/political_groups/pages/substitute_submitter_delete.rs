use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context,
    form::{EmptyForm, Validate},
    political_groups::{self, PoliticalGroup, SubstituteSubmitter},
};

use super::{SubstituteSubmitterDeletePath, SubstituteSubmitterEditPath};

pub async fn delete_substitute_submitter(
    SubstituteSubmitterDeletePath { submitter_id }: SubstituteSubmitterDeletePath,
    political_group: PoliticalGroup,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Ok(
            Redirect::to(&SubstituteSubmitterEditPath { submitter_id }.to_string()).into_response(),
        ),
        Ok(_) => {
            SubstituteSubmitter::delete_by_id(&store, political_group.id, submitter_id).await?;

            Ok(Redirect::to(&political_groups::SubstituteSubmitter::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::extract::Form;

    use crate::{
        AppError, AppStore, Context, TokenValue,
        political_groups::{PoliticalGroupId, SubstituteSubmitter, SubstituteSubmitterId},
        test_utils::{sample_political_group, sample_substitute_submitter},
    };

    #[sqlx::test]
    async fn delete_substitute_submitter_removes_and_redirects(
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
        political_group.update(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { submitter_id },
            political_group,
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
        assert_eq!(location, SubstituteSubmitter::list_path());

        let submitters = PoliticalGroup::list_substitute_submitters(&store, group_id)?;
        assert!(submitters.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_substitute_submitter_invalid_csrf_redirects_to_edit(
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
        political_group.update(&store).await?;

        let context = Context::new_test_without_db();

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { submitter_id },
            political_group,
            context,
            State(store.clone()),
            Form(EmptyForm::new(TokenValue("invalid".to_string()))),
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
        assert_eq!(
            location,
            SubstituteSubmitterEditPath { submitter_id }.to_string()
        );

        let submitters = PoliticalGroup::list_substitute_submitters(&store, group_id)?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }
}

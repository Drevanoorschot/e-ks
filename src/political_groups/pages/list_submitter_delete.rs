use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context,
    form::{EmptyForm, Validate},
    political_groups::{self, ListSubmitter, PoliticalGroup},
};

use super::{ListSubmitterDeletePath, ListSubmitterEditPath};

pub async fn delete_list_submitter(
    ListSubmitterDeletePath { submitter_id }: ListSubmitterDeletePath,
    political_group: PoliticalGroup,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            Ok(Redirect::to(&ListSubmitterEditPath { submitter_id }.to_string()).into_response())
        }
        Ok(_) => {
            ListSubmitter::delete_by_id(&store, political_group.id, submitter_id).await?;

            Ok(Redirect::to(&political_groups::ListSubmitter::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        AppError, AppStore, Context, TokenValue,
        political_groups::{ListSubmitter, ListSubmitterId, PoliticalGroupId},
        test_utils::{sample_list_submitter, sample_political_group},
    };

    #[sqlx::test]
    async fn delete_list_submitter_removes_and_redirects(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();

        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_list_submitter(
            ListSubmitterDeletePath { submitter_id },
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
        assert_eq!(location, ListSubmitter::list_path());

        let submitters = PoliticalGroup::list_submitters(&store, group_id)?;
        assert!(submitters.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_list_submitter_invalid_csrf_redirects_to_edit(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store, political_group.id).await?;

        let context = Context::new_test_without_db();

        let response = delete_list_submitter(
            ListSubmitterDeletePath { submitter_id },
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
        assert_eq!(location, ListSubmitterEditPath { submitter_id }.to_string());

        let submitters = PoliticalGroup::list_submitters(&store, group_id)?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }
}

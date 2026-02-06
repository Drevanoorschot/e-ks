use super::SubstituteSubmittersPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, authorised_agents::AuthorisedAgent, filters,
    list_submitters::ListSubmitter, political_groups::PoliticalGroup,
    substitute_list_submitters::SubstituteSubmitter,
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};

#[derive(Template)]
#[template(path = "substitute_list_submitters/substitute_submitters.html")]
struct SubstituteSubmittersTemplate {
    substitute_submitters: Vec<SubstituteSubmitter>,
}

pub async fn list_substitute_submitters(
    _: SubstituteSubmittersPath,
    context: Context,
    State(store): State<AppStore>,
) -> Result<impl IntoResponse, AppError> {
    let substitute_submitters = store.get_substitute_submitters()?;

    Ok(HtmlTemplate(
        SubstituteSubmittersTemplate {
            substitute_submitters,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};

    use crate::{
        AppError, AppStore, Context,
        political_groups::PoliticalGroupId,
        substitute_list_submitters::SubstituteSubmitterId,
        test_utils::{response_body_string, sample_political_group, sample_substitute_submitter},
    };

    #[sqlx::test]
    async fn list_substitute_submitters_shows_created_submitter(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_group.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let response = list_substitute_submitters(
            SubstituteSubmittersPath {},
            Context::new_test_without_db(),
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
    async fn list_substitute_submitters_shows_edit_link(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_group.create(&store).await?;
        substitute_submitter.create(&store).await?;

        let response = list_substitute_submitters(
            SubstituteSubmittersPath {},
            Context::new_test_without_db(),
            State(store),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&substitute_submitter.edit_path()));

        Ok(())
    }
}

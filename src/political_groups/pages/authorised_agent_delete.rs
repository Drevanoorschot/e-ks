use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context,
    form::{EmptyForm, Validate},
    political_groups::{AuthorisedAgent, PoliticalGroup},
};

use super::{AuthorisedAgentDeletePath, AuthorisedAgentEditPath};

pub async fn delete_authorised_agent(
    AuthorisedAgentDeletePath { agent_id }: AuthorisedAgentDeletePath,
    political_group: PoliticalGroup,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            Ok(Redirect::to(&AuthorisedAgentEditPath { agent_id }.to_string()).into_response())
        }
        Ok(_) => {
            AuthorisedAgent::delete_by_id(&store, political_group.id, agent_id).await?;

            Ok(Redirect::to(&AuthorisedAgent::list_path()).into_response())
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
        political_groups::{AuthorisedAgentId, PoliticalGroup, PoliticalGroupId},
        test_utils::{sample_authorised_agent, sample_political_group},
    };

    #[sqlx::test]
    async fn delete_authorised_agent_removes_and_redirects(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store, political_group.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_authorised_agent(
            AuthorisedAgentDeletePath { agent_id },
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
        assert_eq!(location, AuthorisedAgent::list_path());

        let agents = PoliticalGroup::list_authorised_agents(&store, group_id)?;
        assert!(agents.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_authorised_agent_invalid_csrf_redirects_to_edit(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store, political_group.id).await?;

        let context = Context::new_test_without_db();

        let response = delete_authorised_agent(
            AuthorisedAgentDeletePath { agent_id },
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
        assert_eq!(location, AuthorisedAgentEditPath { agent_id }.to_string());

        let agents = PoliticalGroup::list_authorised_agents(&store, group_id)?;
        assert_eq!(agents.len(), 1);

        Ok(())
    }
}

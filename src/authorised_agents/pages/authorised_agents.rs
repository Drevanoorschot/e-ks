use super::AuthorisedAgentsPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, authorised_agents::AuthorisedAgent, filters,
    list_submitters::ListSubmitter, political_groups::PoliticalGroup,
    substitute_list_submitters::SubstituteSubmitter,
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};

#[derive(Template)]
#[template(path = "authorised_agents/authorised_agents.html")]
struct AuthorisedAgentsTemplate {
    authorised_agents: Vec<AuthorisedAgent>,
}

pub async fn list_authorised_agents(
    _: AuthorisedAgentsPath,
    context: Context,
    State(store): State<AppStore>,
) -> Result<impl IntoResponse, AppError> {
    let authorised_agents = store.get_authorised_agents()?;

    Ok(HtmlTemplate(
        AuthorisedAgentsTemplate { authorised_agents },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;

    use crate::{
        AppError, AppStore, Context,
        authorised_agents::AuthorisedAgentId,
        political_groups::PoliticalGroupId,
        test_utils::{response_body_string, sample_authorised_agent, sample_political_group},
    };

    #[sqlx::test]
    async fn list_authorised_agents_shows_created_agent(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            Context::new_test_without_db(),
            State(store.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorised_agent.last_name));

        Ok(())
    }

    #[sqlx::test]
    async fn list_authorised_agents_shows_edit_link(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            Context::new_test_without_db(),
            State(store.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorised_agent.edit_path()));

        Ok(())
    }
}

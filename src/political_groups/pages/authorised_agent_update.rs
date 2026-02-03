use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{AuthorisedAgent, AuthorisedAgentForm, ListSubmitter, PoliticalGroup},
};

use super::AuthorisedAgentEditPath;

#[derive(Template)]
#[template(path = "political_groups/authorised_agent_update.html")]
struct AuthorisedAgentUpdateTemplate {
    authorised_agents: Vec<AuthorisedAgent>,
    authorised_agent: AuthorisedAgent,
    form: FormData<AuthorisedAgentForm>,
}

pub async fn edit_authorised_agent(
    AuthorisedAgentEditPath { agent_id }: AuthorisedAgentEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
) -> Result<Response, AppError> {
    let authorised_agent =
        PoliticalGroup::get_authorised_agent(&store, political_group.id, agent_id)?;
    let authorised_agents = PoliticalGroup::list_authorised_agents(&store, political_group.id)?;

    Ok(HtmlTemplate(
        AuthorisedAgentUpdateTemplate {
            form: FormData::new_with_data(authorised_agent.clone().into(), &context.csrf_tokens),
            authorised_agent,
            authorised_agents,
        },
        context,
    )
    .into_response())
}

pub async fn update_authorised_agent(
    AuthorisedAgentEditPath { agent_id }: AuthorisedAgentEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
    Form(form): Form<AuthorisedAgentForm>,
) -> Result<Response, AppError> {
    let authorised_agent =
        PoliticalGroup::get_authorised_agent(&store, political_group.id, agent_id)?;
    let authorised_agents = PoliticalGroup::list_authorised_agents(&store, political_group.id)?;

    match form.validate_update(&authorised_agent, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            AuthorisedAgentUpdateTemplate {
                authorised_agent,
                form: form_data,
                authorised_agents,
            },
            context,
        )
        .into_response()),
        Ok(authorised_agent) => {
            authorised_agent.update(&store, political_group.id).await?;

            Ok(Redirect::to(&AuthorisedAgent::list_path()).into_response())
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
        political_groups::{AuthorisedAgent, AuthorisedAgentId, PoliticalGroupId},
        test_utils::{
            response_body_string, sample_authorised_agent, sample_authorised_agent_form,
            sample_political_group,
        },
    };

    #[tokio::test]
    async fn edit_authorised_agent_renders_existing_agent() -> Result<(), AppError> {
        let store = AppStore::default();
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store, political_group.id).await?;

        let response = edit_authorised_agent(
            AuthorisedAgentEditPath { agent_id },
            Context::new_test_without_db(),
            political_group,
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

    #[tokio::test]
    async fn update_authorised_agent_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::default();
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store, political_group.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_authorised_agent(
            AuthorisedAgentEditPath { agent_id },
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
        assert_eq!(location, AuthorisedAgent::list_path());

        let updated = PoliticalGroup::get_authorised_agent(&store, group_id, agent_id)?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_authorised_agent_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::default();
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store, political_group.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_authorised_agent(
            AuthorisedAgentEditPath { agent_id },
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

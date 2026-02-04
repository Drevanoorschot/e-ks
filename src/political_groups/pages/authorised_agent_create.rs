use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use super::AuthorisedAgentNewPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{AuthorisedAgent, AuthorisedAgentForm, ListSubmitter, PoliticalGroup},
};

#[derive(Template)]
#[template(path = "political_groups/authorised_agent_create.html")]
struct AuthorisedAgentCreateTemplate {
    authorised_agents: Vec<AuthorisedAgent>,
    form: FormData<AuthorisedAgentForm>,
}

pub async fn new_authorised_agent_form(
    _: AuthorisedAgentNewPath,
    context: Context,
    State(store): State<AppStore>,
    political_group: PoliticalGroup,
) -> Result<impl IntoResponse, AppError> {
    let authorised_agents = PoliticalGroup::list_authorised_agents(&store, political_group.id)?;

    Ok(HtmlTemplate(
        AuthorisedAgentCreateTemplate {
            authorised_agents,
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_authorised_agent(
    _: AuthorisedAgentNewPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
    Form(form): Form<AuthorisedAgentForm>,
) -> Result<Response, AppError> {
    let authorised_agents = PoliticalGroup::list_authorised_agents(&store, political_group.id)?;

    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            AuthorisedAgentCreateTemplate {
                authorised_agents,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(authorised_agent) => {
            authorised_agent.create(&store, political_group.id).await?;
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
    use sqlx::PgPool;

    use crate::{
        AppError, AppStore, Context,
        political_groups::{AuthorisedAgent, PoliticalGroupId},
        test_utils::{response_body_string, sample_authorised_agent_form, sample_political_group},
    };

    #[sqlx::test]
    async fn new_authorised_agent_form_renders_csrf_field(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let response = new_authorised_agent_form(
            AuthorisedAgentNewPath {},
            Context::new_test_without_db(),
            State(store),
            political_group,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains(&format!("action=\"{}\"", AuthorisedAgent::new_path())));

        Ok(())
    }

    #[sqlx::test]
    async fn create_authorised_agent_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_authorised_agent_form(&csrf_token);

        let response = create_authorised_agent(
            AuthorisedAgentNewPath {},
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

        let agents = PoliticalGroup::list_authorised_agents(&store, group_id)?;
        assert_eq!(agents.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_authorised_agent_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_authorised_agent(
            AuthorisedAgentNewPath {},
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

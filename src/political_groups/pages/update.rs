use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{self, AuthorisedAgent, ListSubmitter, PoliticalGroup, PoliticalGroupForm},
};

use super::PoliticalGroupEditPath;

#[derive(Template)]
#[template(path = "political_groups/update.html")]
struct PoliticalGroupUpdateTemplate {
    form: FormData<PoliticalGroupForm>,
    political_group: PoliticalGroup,
    authorised_agents: Vec<AuthorisedAgent>,
}

pub async fn edit_political_group(
    _: PoliticalGroupEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
) -> Result<Response, AppError> {
    let authorised_agents =
        political_groups::get_authorised_agents(&store, political_group.id).await?;

    Ok(HtmlTemplate(
        PoliticalGroupUpdateTemplate {
            form: FormData::new_with_data(political_group.clone().into(), &context.csrf_tokens),
            political_group,
            authorised_agents,
        },
        context,
    )
    .into_response())
}

pub async fn update_political_group(
    _: PoliticalGroupEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(store): State<AppStore>,
    Form(form): Form<PoliticalGroupForm>,
) -> Result<Response, AppError> {
    let authorised_agents =
        political_groups::get_authorised_agents(&store, political_group.id).await?;

    match form.validate_update(&political_group, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PoliticalGroupUpdateTemplate {
                form: form_data,
                political_group,
                authorised_agents,
            },
            context,
        )
        .into_response()),
        Ok(political_group) => {
            political_groups::update_political_group(&store, &political_group).await?;

            Ok(Redirect::to(&PoliticalGroup::edit_path()).into_response())
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
        political_groups::{self, AuthorisedAgentId, PoliticalGroupId},
        test_utils::{
            response_body_string, sample_authorised_agent, sample_political_group,
            sample_political_group_form,
        },
    };

    #[tokio::test]
    async fn edit_political_group_renders_existing_data() -> Result<(), AppError> {
        let store = AppStore::default();
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&store, &political_group).await?;
        political_groups::create_authorised_agent(&store, political_group.id, &authorised_agent)
            .await?;

        let response = edit_political_group(
            PoliticalGroupEditPath {},
            Context::new_test_without_db(),
            political_group,
            State(store),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("Kiesraad Demo"));
        assert!(body.contains(&authorised_agent.last_name));

        Ok(())
    }

    #[tokio::test]
    async fn update_political_group_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::default();
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&store, &political_group).await?;
        political_groups::create_authorised_agent(&store, political_group.id, &authorised_agent)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_political_group_form(&csrf_token, Some(agent_id));

        let response = update_political_group(
            PoliticalGroupEditPath {},
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
        assert_eq!(location, PoliticalGroup::edit_path());

        let updated =
            political_groups::get_single_political_group(&store)?.expect("political group");
        assert_eq!(updated.long_list_allowed, Some(true));
        assert_eq!(updated.display_name_confirmed, Some(true));
        assert_eq!(updated.legal_name_confirmed, Some(true));
        assert_eq!(updated.authorised_agent_id, Some(agent_id));

        Ok(())
    }

    #[tokio::test]
    async fn update_political_group_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::default();
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&store, &political_group).await?;
        political_groups::create_authorised_agent(&store, political_group.id, &authorised_agent)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_political_group_form(&csrf_token, None);
        form.authorised_agent_id = "not-a-uuid".to_string();

        let response = update_political_group(
            PoliticalGroupEditPath {},
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
        assert!(body.contains("The provided value is not valid."));

        Ok(())
    }
}

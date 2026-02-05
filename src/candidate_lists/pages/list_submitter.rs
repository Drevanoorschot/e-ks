use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    candidate_lists::{
        CandidateList, CandidateListSummary, ListSubmitterForm, pages::EditListSubmitterPath,
    },
    filters,
    form::{FormData, Validate},
    persons::Person,
    political_groups::{ListSubmitter, PoliticalGroup},
};

#[derive(Template)]
#[template(path = "candidate_lists/list_submitter.html")]
struct ListSubmitterUpdateTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: usize,
    form: FormData<ListSubmitterForm>,
    candidate_list: CandidateList,
    list_submitters: Vec<ListSubmitter>,
}

pub async fn edit_list_submitter_form(
    _: EditListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
) -> Result<Response, AppError> {
    let candidate_lists = CandidateListSummary::get(&store)?;
    let total_persons = store.get_person_count()?;
    let list_submitters = PoliticalGroup::list_submitters(&store, context.political_group.id)?;

    let form = FormData::new_with_data(
        ListSubmitterForm::from(candidate_list.clone()),
        &context.csrf_tokens,
    );

    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            candidate_lists,
            total_persons,
            form,
            candidate_list,
            list_submitters,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter(
    _: EditListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let candidate_lists = CandidateListSummary::get(&store)?;
    let total_persons = store.get_person_count()?;
    let list_submitters = PoliticalGroup::list_submitters(&store, context.political_group.id)?;
    match form.validate_update(&candidate_list, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterUpdateTemplate {
                candidate_lists,
                total_persons,
                form: form_data,
                candidate_list,
                list_submitters,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
            let candidate_list = candidate_list.update_list_submitter(&store).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{StatusCode, header};
    use axum_extra::extract::Form;
    use chrono::DateTime;

    use crate::{
        AppStore, Context, CsrfTokens, ElectoralDistrict, Locale, TokenValue,
        candidate_lists::CandidateListId,
        political_groups::{ListSubmitterId, PoliticalGroupId},
        test_utils::{
            response_body_string, sample_candidate_list, sample_list_submitter,
            sample_political_group,
        },
    };

    #[sqlx::test]
    async fn edit_list_submitter_renders_list_submitter_form(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let political_group = sample_political_group(PoliticalGroupId::new());

        candidate_list.create(&store).await?;
        political_group.create(&store).await?;
        list_submitter.create(&store, political_group.id).await?;

        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());

        let response = edit_list_submitter_form(
            EditListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));
        assert!(body.contains("csrf_token"));
        assert!(body.contains(&candidate_list.edit_list_submitter_path()));
        assert!(body.contains(&list_submitter.last_name));
        assert!(body.contains(&list_submitter.initials));

        Ok(())
    }

    #[sqlx::test]
    async fn update_list_submitter_persists_and_redirects(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let political_group = sample_political_group(PoliticalGroupId::new());
        political_group.create(&store).await?;
        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());
        let csrf_token = context.csrf_tokens.issue().value;
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            list_submitter_id: None,
            candidates: vec![],
            created_at: creation_date,
            updated_at: creation_date,
        };
        let list_submitter = sample_list_submitter(ListSubmitterId::new());

        candidate_list.create(&store).await?;
        list_submitter.create(&store, political_group.id).await?;

        let form = ListSubmitterForm {
            list_submitter_id: list_submitter.id.to_string(),
            csrf_token,
        };
        let response = update_list_submitter(
            EditListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store.clone()),
            Form(form),
        )
        .await
        .unwrap();

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        // verify updated candidate list object in database
        let lists = CandidateListSummary::get(&store)?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        assert_eq!(updated_list.view_path(), location);

        assert_eq!(candidate_list.id, updated_list.id);

        assert_eq!(list_submitter.id, updated_list.list_submitter_id.unwrap());

        Ok(())
    }

    #[sqlx::test]
    async fn update_list_submitter_invalid_form_renders_template(
        pool: sqlx::PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let political_group = sample_political_group(PoliticalGroupId::new());
        political_group.create(&store).await?;
        let context = Context::new(political_group.clone(), Locale::En, CsrfTokens::default());
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());

        candidate_list.create(&store).await?;
        list_submitter.create(&store, political_group.id).await?;

        let form = ListSubmitterForm {
            list_submitter_id: list_submitter.id.to_string(),
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update_list_submitter(
            EditListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store.clone()),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));

        let lists = CandidateListSummary::get(&store)?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        // verify candidate list didn't update in database
        assert_eq!(
            candidate_list.electoral_districts,
            updated_list.electoral_districts
        );

        Ok(())
    }
}

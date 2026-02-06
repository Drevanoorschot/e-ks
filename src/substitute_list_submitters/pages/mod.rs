use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{
    AppError, AppState,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

mod substitute_submitter_create;
mod substitute_submitter_delete;
mod substitute_submitter_update;
mod substitute_submitters;

#[derive(TypedPath, Deserialize)]
#[typed_path("/political-group/substitute-submitters", rejection(AppError))]
pub struct SubstituteSubmittersPath;

#[derive(TypedPath)]
#[typed_path("/political-group/substitute-submitters/new", rejection(AppError))]
pub struct SubstituteSubmitterNewPath;

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/substitute-submitters/{sub_submitter_id}/edit",
    rejection(AppError)
)]
pub struct SubstituteSubmitterEditPath {
    pub sub_submitter_id: SubstituteSubmitterId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/political-group/substitute-submitters/{sub_submitter_id}/delete",
    rejection(AppError)
)]
pub struct SubstituteSubmitterDeletePath {
    pub sub_submitter_id: SubstituteSubmitterId,
}

impl SubstituteSubmitter {
    pub fn list_path() -> String {
        SubstituteSubmittersPath {}.to_string()
    }

    pub fn new_path() -> String {
        SubstituteSubmitterNewPath {}.to_string()
    }

    pub fn edit_path(&self) -> String {
        SubstituteSubmitterEditPath {
            sub_submitter_id: self.id,
        }
        .to_string()
    }

    pub fn delete_path(&self) -> String {
        SubstituteSubmitterDeletePath {
            sub_submitter_id: self.id,
        }
        .to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(substitute_submitters::list_substitute_submitters)
        .typed_get(substitute_submitter_create::new_substitute_submitter_form)
        .typed_post(substitute_submitter_create::create_substitute_submitter)
        .typed_get(substitute_submitter_update::edit_substitute_submitter)
        .typed_post(substitute_submitter_update::update_substitute_submitter)
        .typed_post(substitute_submitter_delete::delete_substitute_submitter)
}

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

use crate::{AppError, AppState, political_groups::PoliticalGroup};

mod update;

#[derive(TypedPath)]
#[typed_path("/political-group", rejection(AppError))]
pub struct PoliticalGroupEditPath;

impl PoliticalGroup {
    pub fn edit_path() -> String {
        PoliticalGroupEditPath {}.to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(update::edit_political_group)
        .typed_post(update::update_political_group)
}

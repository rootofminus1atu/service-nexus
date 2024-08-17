use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use axum::http::StatusCode;
use tracing::error;

use self::model::Cat;

mod model;
mod controller;
mod rarities;
mod names;


pub fn routes(db: mongodb::Database) -> Router {
    let cats = db.collection::<Cat>("cats");

    Router::new()
        .route("/", get(self::controller::get_all))
        .route("/:id", get(self::controller::get_one))
        .route("/random", post(self::controller::get_random))
        .with_state(cats)
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("MongoDB cats error: {0}")]
    CatDbError(#[from] mongodb::error::Error),
    #[error("Cat reqwest error: {0}")]
    CatReqwestError(#[from] reqwest::Error),
    #[error("Cat with id {id} not found")]
    NotFound { id: String },
    #[error("No cat was delivered from RandomCatAPI")]
    NoCatsFromRandomCatApi,
    #[error("No breeds were delivered from RandomCatAPI")]
    NoBreedsFromRandomCatApi,
    #[error("No random user was delievered from RandomUserAPI")]
    NoPeopleFromRandomUserApi,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        error!("->> {}", self);

        let body = Json(serde_json::json!({
            "error": format!("{}", &self)
        }));

        let status_code = match &self {
            Self::NotFound { id: _ } => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        };
        
        (status_code, body).into_response()
    }
}


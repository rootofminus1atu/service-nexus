use std::sync::Arc;
use axum::response::Redirect;
use axum::{response::IntoResponse, Extension, Json, Router, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use tracing::error;
use utoipa::openapi;
use utoipa::ToSchema;
use utoipa::OpenApi;
use self::helpers::utoipa_ext::nest_openapis_at_prefix;

use super::SupabaseResources;

mod quotes;
mod images;
mod helpers;


pub fn routes(supabase: Arc<SupabaseResources>) -> Router {
    let openapi = build_openapi();

    Router::new()   
        .nest("/quotes", quotes::routes())
        .nest("/images", images::routes())
        .nest_service("/", ServeDir::new("public"))
        .route("/doc.json", get(|| async { Json(openapi) } ))
        .route("/", get(|| async { Redirect::permanent("/index.html") }))
        .layer(Extension(supabase))
}


pub fn build_openapi() -> utoipa::openapi::OpenApi {
    let quotes_api = quotes::QuotesApi::openapi();
    let images_api = images::ImagesApi::openapi();

    nest_openapis_at_prefix(quotes_api, images_api, "/jp2")
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CountResponse {
    pub count: i64
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Supabase jp2 error: {0}")]
    SupabaseJp2Error(#[from] sqlx::Error),
    #[error("Quote with id {id} not found")]
    QuoteWithIdNotFound { id: i64 },
    #[error("Invalid quote id passed: {id}")]
    InvalidQuoteId { id: String },
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        error!("->> {}", self);

        let body = Json(serde_json::json!({
            "error": format!("{}", &self)
        }));

        let status_code = match &self {
            Self::QuoteWithIdNotFound { id: _ } => StatusCode::NOT_FOUND,
            Self::InvalidQuoteId { id: _ } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        };
        
        (status_code, body).into_response()
    }
}
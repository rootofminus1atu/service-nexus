use std::sync::Arc;
use axum::{response::IntoResponse, routing::get, Extension, Json, Router};
use sqlx::{query_as, query_scalar};
use utoipa::OpenApi;

use crate::web::{Object, SupabaseResources};

use super::CountResponse;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_all))
        .route("/random", get(get_random))
        .route("/count", get(get_count))
}

#[derive(OpenApi)]
#[openapi(
    paths(get_all, get_random),
    // components(schemas(??? should be the img object but links will do for now)),
    tags(
        (name = "images", description = "Images API endpoints")
    )
)]
pub struct ImagesApi;

#[utoipa::path(
    get,
    path = "/",
    tag = "images",
    responses(
        (status = 200, description = "Get all images", body = [String])
    )
)]
async fn get_all(Extension(supabase): Extension<Arc<SupabaseResources>>) -> Result<impl IntoResponse, super::Error> {
    let img_links = query_as::<_, Object>("SELECT * FROM storage.objects WHERE bucket_id = $1 AND metadata->>'mimetype' != 'application/octet-stream'")
        .bind(&supabase.storage.bucket_id)
        .fetch_all(&supabase.db)
        .await?
        .iter()
        .map(|img| img.to_link(&supabase.storage))
        .collect::<Vec<_>>();

    Ok(Json(img_links))
}

#[utoipa::path(
    get,
    path = "/random",
    tag = "images",
    responses(
        (status = 200, description = "Get a random image", body = String)
    )
)]
async fn get_random(Extension(supabase): Extension<Arc<SupabaseResources>>) -> Result<impl IntoResponse, super::Error> {
    let img_link = query_as::<_, Object>("SELECT * FROM storage.objects WHERE bucket_id = $1 AND metadata->>'mimetype' != 'application/octet-stream' ORDER BY RANDOM() LIMIT 1")
        .bind(&supabase.storage.bucket_id)
        .fetch_one(&supabase.db)
        .await?
        .to_link(&supabase.storage);

    Ok(Json(img_link))
}


async fn get_count(Extension(supabase): Extension<Arc<SupabaseResources>>) -> Result<impl IntoResponse, super::Error> {
    let count = query_scalar("SELECT COUNT(*) FROM storage.objects WHERE bucket_id = $1 AND metadata->>'mimetype' != 'application/octet-stream'")
        .bind(&supabase.storage.bucket_id)
        .fetch_one(&supabase.db)
        .await?;

    Ok(Json(CountResponse { count }))
}
use std::sync::Arc;
use axum::{Extension, Router};
use serde::{Deserialize, Serialize};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tower_http::cors::{self, CorsLayer};
use utoipa::ToSchema;
use tracing::info;

mod cats;
mod timetable;
mod jp2;

/// ## senv = shuttle env
/// Reads a shuttle secret from `SecretStore`, crashes the program if an env var with that name is not found.
macro_rules! senv {
    ($store:expr, $key:ident) => {{
        use shuttle_runtime::__internals::Context;
        $store.get(stringify!($key)).context(concat!("could not get ", stringify!($key)))?
    }};
}

// macro_rules! senv_multiple {
//     ($store:expr, $( $key:ident ),+ ) => {{
//         use shuttle_runtime::__internals::Context;
//         (
//             $(
//                 $store.get(stringify!($key)).context(concat!("could not get ", stringify!($key)))?
//             ),+
//         )
//     }};
// }

pub async fn setup_web_server(secret_store: &SecretStore) -> Result<Router, shuttle_runtime::Error> {
    let cat_api_key = senv!(secret_store, CAT_API_KEY);
    let mongo_uri = senv!(secret_store, MONGO_URI);
    let database_url = senv!(secret_store, DATABASE_URL);
    let supabase_url = senv!(secret_store, SUPABASE_URL);


    let mongo_client = mongodb::Client::with_uri_str(mongo_uri).await
        .map_err(|e| shuttle_runtime::Error::Database(format!("could not connect to mongo: {}", e)))?;
    info!("connected to mongo");
    let mongo_db = mongo_client.database("unboxcat");

    let supabase_db = PgPool::connect(&database_url).await
        .map_err(|e| shuttle_runtime::Error::Database(format!("could not connect to supabase: {}", e)))?;
    info!("connected to postgres");
    let supabase_storage = Storage::new("cenzo", &supabase_url);
    let supabase = Arc::new(SupabaseResources::new(supabase_db, supabase_storage));
    info!("created supabase");

    let client = ClientWithKeys::new(cat_api_key);
    info!("created new reqwest client");

    let router = Router::new()
        .nest("/cats", self::cats::routes(mongo_db))
        .nest("/timetable", self::timetable::routes())
        .nest("/jp2", self::jp2::routes(supabase))
        .layer(Extension(client))
        .layer(CorsLayer::new()
            .allow_origin(cors::Any)
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_credentials(true)
        );

    Ok(router)
}

#[derive(Debug, Clone)]
pub struct ClientWithKeys {
    client: reqwest::Client,
    cat_api_key: Arc<String>
}

impl ClientWithKeys {
    pub fn new(cat_api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            cat_api_key: Arc::new(cat_api_key)
        }
    }
}


#[derive(Debug, Clone)]
pub struct SupabaseResources {
    pub db: PgPool,
    pub storage: Storage,
}

impl SupabaseResources {
    pub fn new(db: PgPool, storage: Storage) -> Self {
        Self { db, storage }
    }
}


#[derive(Debug, Clone)]
pub struct Storage {
    pub bucket_id: String,
    pub supabase_url: String,
    // future props for api key etc
}

impl Storage {
    pub fn new(bucket_id: &str, supabase_url: &str) -> Self {
        Storage { bucket_id: bucket_id.to_string(), supabase_url: supabase_url.to_string() }
    }
}


#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, ToSchema)]
pub struct Object {
    name: String,
    bucket_id: String
}

impl Object {
    pub fn to_link(&self, storage_data: &Storage) -> String {
        format!("{}/storage/v1/object/public/{}/{}", storage_data.supabase_url, storage_data.bucket_id, self.name)
    }
}
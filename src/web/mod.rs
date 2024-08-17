use std::sync::Arc;
use axum::{Extension, Router};
use shuttle_runtime::SecretStore;
use shuttle_runtime::__internals::Context;
use tracing::info;

mod cats;

pub async fn setup_web_server(secret_store: &SecretStore) -> Result<Router, shuttle_runtime::Error> {
    let cat_api_key = secret_store.get("CAT_API_KEY")
        .context("could not get cat api key")?;
    let mongo_uri = secret_store.get("MONGO_URI")
        .context("could not get mongo uri")?;


    let mongo_client = mongodb::Client::with_uri_str(mongo_uri).await
        .map_err(|e| shuttle_runtime::Error::Database(format!("could not connect to mongo: {}", e)))?;

    info!("connected to mongo");
        // .map_err(|e| shuttle_runtime::Error::Database(format!("could not parse mongo uri: {}", e)))?;
    let db = mongo_client.database("unboxcat");

    let client = ClientWithKeys::new(cat_api_key);

    info!("created new reqwest client");

    let router = Router::new()
        .nest("/cats", self::cats::routes(db))
        .layer(Extension(client));

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
use axum::Router;
use shuttle_runtime::SecretStore;
use axum::routing::get;



pub async fn setup_web_server(_secret_store: &SecretStore) -> Result<Router, shuttle_runtime::Error> {
    
    let router = Router::new().route("/", get(|| async { "Hello from Axum!" }));

    Ok(router)
}
use axum::middleware::{from_fn, from_fn_with_state};
use axum::Router;
use axum::routing::{get, post, put};
use sqlx::PgPool;

mod controller;
mod model;
mod error;
mod auth;

use error::Error;

pub fn routes(db: PgPool) -> Router {
    let public_routes = Router::new()
        .route("/weapons", get(controller::get_all_weapons))
        .route("/weapons/:id", get(controller::get_weapon))
        .route("/loadouts", get(controller::get_all_loadouts))
        .route("/ladouts/:id", get(controller::get_loadout));

    let auth_routes = Router::new()
        .route("/loadouts", post(controller::create_loadout));

    let ownership_routes = Router::new()
        .route("/loadouts/:id", put(controller::update_loadout).delete(controller::delete_loadout));

    Router::new()
        .merge(public_routes)
        .merge(auth_routes.layer(from_fn(auth::auth_mw)))
        .merge(ownership_routes.layer(from_fn_with_state(db.clone(), auth::loadout_ownership_mw)).layer(from_fn(auth::auth_mw)))  // i think the order has to be reversed like this for auth to be applied first
        .with_state(db)
}

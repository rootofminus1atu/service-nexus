use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

mod controller;
mod model;
mod error;

use error::Error;

pub fn routes(db: PgPool) -> Router {
    Router::new()
        .route("/weapons", get(self::controller::get_all_weapons))
        .route("/weapons/:id", get(self::controller::get_weapon))
        .route("/loadouts", 
            get(self::controller::get_all_loadouts)
            .post(self::controller::create_loadout)
        )
        .route("/loadouts/:id", 
            get(self::controller::get_loadout)
            .delete(self::controller::delete_loadout)
            .put(self::controller::update_loadout)
        )
        .with_state(db)
}




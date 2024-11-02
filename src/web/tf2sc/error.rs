use axum::response::IntoResponse;
use axum::Json;
use axum::http::StatusCode;
use sqlx::types::Uuid;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // 500s
    #[error("Neon tf2sc error: {0}")]
    NeonTf2scError(#[from] sqlx::Error),

    // 400s
    #[error("Invalid weapon id")]
    InvalidWeaponId,
    #[error("Weapon with id {id} not found")]
    WeaponNotFound { id: i32 },
    #[error("Invalid loadout id")]
    InvalidLoadoutId,
    #[error("Loadout with id {id} not found")]
    LoadoutNotFound { id: Uuid }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        error!("->> {}", self);

        let body = Json (serde_json::json!({
            "error": format!("{}", &self)
        }));

        let status_code = match &self {
            Self::WeaponNotFound { id: _ } | Self::LoadoutNotFound { id: _ } => StatusCode::NOT_FOUND,
            Self::InvalidWeaponId | Self::InvalidLoadoutId => StatusCode::BAD_REQUEST,
            Self::NeonTf2scError(_) => StatusCode::INTERNAL_SERVER_ERROR
        };
        
        (status_code, body).into_response()
    }
}

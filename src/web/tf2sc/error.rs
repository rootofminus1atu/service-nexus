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
    LoadoutNotFound { id: Uuid },
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("Auth Error: {0}")]
    AuthError(#[from] AuthError),
    #[error("You don't own this resource")]
    NotOwned
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        error!("->> {}", self);

        let body = Json (serde_json::json!({
            "error": format!("{}", &self)
        }));

        let status_code = match &self {
            Self::WeaponNotFound { id: _ } | Self::LoadoutNotFound { id: _ } => StatusCode::NOT_FOUND,
            Self::InvalidWeaponId | Self::InvalidLoadoutId | Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::NeonTf2scError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AuthError(err) => match err {
                AuthError::MissingToken => StatusCode::FORBIDDEN,
                _ => StatusCode::FORBIDDEN
            },
            Self::NotOwned => StatusCode::UNAUTHORIZED
        };
        
        (status_code, body).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing Token")]
    MissingToken,
    #[error("Missing Header")]
    MissingHeader,
    #[error("Invalid Header")]
    InvalidHeader,
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Couldn't fetch auth stuff")]
    FetchError,
    #[error("Key Mismatch")]
    KeyMismatch
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        error!("->> {}", self);

        let body = Json (serde_json::json!({
            "error": "Unauthorized"
        }));

        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}



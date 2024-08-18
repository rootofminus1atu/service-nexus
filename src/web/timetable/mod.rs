use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use tracing::error;

mod controller;
mod parsing;
mod url;
mod weekday;


pub fn routes() -> Router {
    Router::new()
        .route("/lessons", post(controller::get_lessons))
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Weekday error: {0}")]
    WeekDayError(#[from] self::weekday::WeekDayError),
    #[error("Parsing error: {0}")]
    ParsingError(#[from] self::parsing::ParsingError),
    #[error("Timetable reqwest error: {0}")]
    TimetableReqwestError(#[from] reqwest::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        error!("->> {}", self);

        let body = Json(serde_json::json!({
            "error": format!("{}", &self)
        }));

        let status_code = match &self {
            _ => StatusCode::INTERNAL_SERVER_ERROR
        };
        
        (status_code, body).into_response()
    }
}





use axum::{response::IntoResponse, Extension, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::web::{timetable::{parsing::get_all_lessons, url::TimetableUrl, weekday::get_week_number_for_date}, ClientWithKeys};

use super::parsing::Lesson;



// TODO: 
// - better error handling at the function_handler level
// - better error reporting at the scraper level, with some parsing/validation entry and some lifetime structs
// - allow to return poisoned timetables, aka ones with mising data chunks

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestBody {
    #[serde(rename = "timetableId")]
    timetable_id: String,
    date: NaiveDate
}

#[derive(Debug, Serialize)]
struct ResponseBody {
    lessons: Vec<Lesson>
}

pub async fn get_lessons(Extension(client): Extension<ClientWithKeys>, Json(payload): Json<RequestBody>) -> Result<impl IntoResponse, super::Error> {
    let RequestBody { timetable_id, date } = payload;

    let week_number = get_week_number_for_date(date, client.client.clone()).await?;
    let url = TimetableUrl::default(timetable_id.clone(), week_number).construct();
    info!("url: {}", url);

    let html = client.client.get(&url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let lessons = get_all_lessons(&html)?;

    let body = ResponseBody { lessons };

    Ok(Json(body))
}
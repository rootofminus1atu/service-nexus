use axum::response::IntoResponse;
use axum::{Extension, Json, Router};
use axum::routing::get;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use chrono::Utc;
use std::sync::LazyLock;

use super::ClientWithKeys;



pub fn routes(client: ClientWithKeys) -> Router {
    let history: LocationHistory = Arc::new(Mutex::new(Vec::new()));
    LazyLock::force(&ROUTES);

    let history_clone = history.clone();
    
    tokio::spawn(async move {
        loop {
            match get_location(&client).await {
                Ok(record) => {
                    let mut list = history_clone.lock().await;
                    list.push(record);

                    let max_list_len = 720;
                    let list_len = list.len();

                    if list.len() > max_list_len {
                        list.drain(0..(list_len - max_list_len));
                    }
                },
                Err(e) => {
                    println!("ERROR: {}\n{:?}", e, e);
                }
            }
            println!("eeping");
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    Router::new()
        .route("/current", get(get_current))
        .route("/history", get(get_history))
        .layer(Extension(history))
}

type LocationHistory = Arc<Mutex<Vec<Record>>>;

pub static ROUTES: LazyLock<HashMap<String, RouteInfo>> = LazyLock::new(|| {
    println!("NEW DEBUG STUFF");
    let assets_dir = std::path::Path::new("assets");
    if let Ok(entries) = std::fs::read_dir(assets_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("Found file: {:?}", entry.file_name());
            }
        }
    }

    let cwd = std::env::current_dir().unwrap();
    let file_path = cwd.join("assets").join("routes.txt");
    let contents = std::fs::read_to_string(file_path).expect("Failed to read routes file");
    println!("AFTER NEW STUFF");


    let cwd = std::env::current_dir().unwrap();
    println!("Current working directory: {:?}", cwd);
    for entry in std::fs::read_dir(&cwd).unwrap() {
        let entry = entry.unwrap();
        println!(" - {:?}", entry.file_name());
    }

    let contents = std::fs::read_to_string("assets/routes.txt").expect("Failed to read routes file");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(contents.as_bytes());

    rdr.records()
        .filter_map(|result| {
            result.ok().and_then(|rec| {
                let route_id = rec.get(0)?.to_string();
                let short_name = rec.get(2)?.to_string();
                let long_name = rec.get(3)?.to_string();
                Some(RouteInfo { route_id, short_name, long_name })
            })
        })
        .map(|r| (r.route_id.clone(), r))
        .collect()
});

async fn get_current(Extension(client): Extension<ClientWithKeys>) -> Result<impl IntoResponse, Error> {
    get_location(&client).await.map(Json)
}

async fn get_history(Extension(history): Extension<LocationHistory>) -> Result<impl IntoResponse, Error> {
    let list = history.lock().await.clone();
    Ok(Json(list))
}

async fn get_location(client: &ClientWithKeys) -> Result<Record, Error> {
    let res = client.client.get("https://api.nationaltransport.ie/gtfsr/v2/Vehicles?format=json")
        .header("x-api-key", client.bus_api_key.as_str())
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(status) = res.get("statusCode").and_then(|v| v.as_u64()) {
        if status == 429 {
            let msg = res.get("message").and_then(|m| m.as_str()).unwrap_or("Rate limit exceeded").to_string();
            return Err(Error::RateLimited(msg));
        }
    }

    let pretty = serde_json::to_string_pretty(&res).unwrap_or_default();
    for (i, line) in pretty.lines().take(20).enumerate() {
        println!("{:02}: {}", i + 1, line);
    }
    let res: Res = serde_json::from_value(res)?;


    let route_ids = ["5265_118111"];
    let route_id = route_ids[0];
    
    let entity = res.entity.iter().find(|e| e.vehicle.trip.route_id == route_id).expect("damn it doesnt exist wtf");
    dbg!(&entity);
    
    let locations = res.entity.iter()
        .filter(|e| route_ids.contains(&e.vehicle.trip.route_id.as_str()))
        .map(|e| Location {
            lat: e.vehicle.position.latitude, 
            lon: e.vehicle.position.longitude,
            ts: e.vehicle.timestamp.clone(),
            route: ROUTES.get(&e.vehicle.trip.route_id).unwrap().clone(),
            vehicle_id: e.vehicle.vehicle.id.clone()
        })
        .collect::<Vec<_>>();

    for loc in &locations {
        let link = format!("[{} - {}]: https://www.google.com/maps?q={},{}", loc.route.route_id, loc.route.short_name, loc.lat, loc.lon);
        println!("{}", link);
    }

    let record = Record {
        ts: Utc::now().timestamp().to_string(),
        locations
    };

    Ok(record)
}



#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Some error")]
    SomeError,
    #[error("Cat reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Rate limited: {0}")]
    RateLimited(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("->> {}", self);

        let body = Json(serde_json::json!({
            "error": format!("{}", &self)
        }));

        let status_code = match &self {
            _ => StatusCode::INTERNAL_SERVER_ERROR
        };
        
        (status_code, body).into_response()
    }
}



#[derive(Serialize, Debug, Clone)]
struct Record {
    ts: String,
    locations: Vec<Location>
}


#[derive(Serialize, Debug, Clone)] 
struct Location {
    lat: f64,
    lon: f64,
    ts: String,
    route: RouteInfo,
    vehicle_id: String
}



#[derive(Deserialize, Debug, Clone)]
struct Res {
    entity: Vec<Entity>
}

#[derive(Deserialize, Debug, Clone)]
struct Entity {
    id: String,
    vehicle: Vehicle
}

#[derive(Deserialize, Debug, Clone)]
struct Vehicle {
    trip: Trip,
    timestamp: String,
    position: Position,
    vehicle: VehicleDetails
}

#[derive(Deserialize, Debug, Clone)]
struct VehicleDetails {
    id: String
}

#[derive(Deserialize, Debug, Clone)]
struct Trip {
    route_id: String
}

#[derive(Deserialize, Debug, Clone)]
struct Position {
    latitude: f64,
    longitude: f64
}

#[derive(Serialize, Debug, Clone)]
pub struct RouteInfo {
    route_id: String,
    short_name: String,
    long_name: String
}

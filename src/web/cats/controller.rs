use std::time::Instant;
use axum::{extract::{Path, State}, response::IntoResponse, Extension, Json};
use poise::serenity_prelude::futures::TryStreamExt;
use mongodb::{bson::doc, Collection};
use tracing::info;
use crate::web::{cats::model::{Cat, CatUnprocessed}, ClientWithKeys};
use super::{model::CatForCreate, names::{get_random_full_name, get_random_name_from_country}, rarities::get_random_rarity};


pub async fn get_all(State(cats): State<Collection<Cat>>) -> Result<impl IntoResponse, super::Error> {
    let cursor = cats
        .find(doc! {})
        .sort(doc! { "createdAt": -1 }).await?;

    let cats: Vec<Cat> = cursor.try_collect().await?;

    Ok(Json(cats))
}

pub async fn get_one(Path(id): Path<String>, State(cats): State<Collection<Cat>>) -> Result<impl IntoResponse, super::Error> {
    let cat = cats
        .find_one(doc! { "_id": &id }).await?
        .ok_or(super::Error::NotFound { id })?;

    Ok(Json(cat))
}

pub async fn get_random(State(cats): State<Collection<Cat>>, Extension(client): Extension<ClientWithKeys>) -> Result<impl IntoResponse, super::Error> {
    info!("=== /cats/random start ===");

    let start_time = Instant::now();
    let response = client.client
        .get(format!("https://api.thecatapi.com/v1/images/search?api_key={}&has_breeds=1", client.cat_api_key.clone()))
        .send()
        .await?;

    let response_text = response.text().await?;
    info!("Response text: {}", response_text);

    let cat: Vec<CatUnprocessed> = serde_json::from_str(&response_text)?;
    let cat = cat.into_iter()
        .next()  // because this req returns a vec, not a single cat, its always 1 cat anyway, or should be
        .ok_or(super::Error::NoCatsFromRandomCatApi)?;

    // let cat = client.client
    //     .get(format!("https://api.thecatapi.com/v1/images/search?api_key={}&has_breeds=1", client.cat_api_key.clone()))
    //     .send()
    //     .await?
    //     .json::<Vec<CatUnprocessed>>()
    //     .await?
    //     .into_iter()
    //     .next()  // because this req returns a vec, not a single cat, its always 1 cat anywa, or should be
    //     .ok_or(super::Error::NoCatsFromRandomCatApi)?;
    info!("fetching catapi: {:?}", start_time.elapsed());
    // look it up in db
    
    let start_time = Instant::now();
    if let Some(found) = cats.find_one(doc! { "_id": &cat.id }).await? {
        info!("Cat that was previously discovered! {:?}\n{} - {}", found, found.full_name, found.rarity);
        return Ok(Json(found.into()))  // here cat turns into a cat_for_create, because of how this lib's sillyness
    }
    info!("checking in mongo: {:?}", start_time.elapsed());
    // if one doesnt exist in it yet, create a new cat

    let cat_wip = cat.start_processing()?;

    let rarity = get_random_rarity();
    let start_time = Instant::now();
    let pet_name = get_random_name_from_country(&cat_wip.breed.country_code, client).await?;
    info!("constructing pet_name: {:?}", start_time.elapsed());
    let full_name = get_random_full_name(&cat_wip.breed, &pet_name);

    let cats_for_create: Collection<CatForCreate> = cats.clone_with_type();

    // NOTE: 
    // becuase the mongodb crate is stupid theres no createdAt field, so i just have to generate a time here, which will be somewhat inaccurate, but whatever
    let new_cat = cat_wip.finalize_processing(rarity.into(), pet_name.into(), full_name.into());

    // for some reason this lib only returns the inserted_id
    let start_time = Instant::now();
    let _insert_res = cats_for_create.insert_one(&new_cat).await?;
    info!("inserting into mongo: {:?}", start_time.elapsed());
    // so i might just return the cat_for_create

    Ok(Json(new_cat))
}
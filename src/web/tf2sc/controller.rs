use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Deserializer};
use sqlx::{types::Uuid, PgPool};
use std::str::FromStr;
use serde::de;
use super::model::{ItemSlot, Loadout, LoadoutForCreate, LoadoutForUpdate, Merc, WeaponFromView};

#[derive(Deserialize)]
pub struct MercSlotParams {
    merc: Option<Merc>,
    item_slot: Option<ItemSlot>,
}

#[allow(unused)]
fn case_insensitive_option_merc<'de, D>(deserializer: D) -> Result<Option<Merc>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|s| Merc::from_str(&s.to_lowercase()).map_err(de::Error::custom))
        .transpose()
}


pub async fn get_all_weapons(State(db): State<PgPool>, Query(q): Query<MercSlotParams>) -> Result<impl IntoResponse, super::Error> {
    let MercSlotParams { 
        merc, 
        item_slot
    }  = q;

    let query = match (merc, item_slot) {
        (None, None) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details")
        },
        (Some(merc), None) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details WHERE merc = $1 OR merc IS NULL")
                .bind(merc)
        },
        (None, Some(item_slot)) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details WHERE item_slot = $1")
                .bind(item_slot)
        },
        (Some(merc), Some(item_slot)) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details WHERE (merc = $1 OR merc IS NULL) AND item_slot = $2")
                .bind(merc)
                .bind(item_slot)
        },
    };

    let weapons = query.fetch_all(&db).await?;

    Ok(Json(weapons))
}

pub async fn get_weapon(Path(id): Path<String>, State(db): State<PgPool>) -> Result<impl IntoResponse, super::Error> {
    let id = id.parse::<i32>()
        .map_err(|_| super::Error::InvalidWeaponId)?;
    
    let weapon = sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(super::Error::WeaponNotFound { id })?;
    
    Ok(Json(weapon))
}

pub async fn get_all_loadouts(State(db): State<PgPool>) -> Result<impl IntoResponse, super::Error> {
    let loadouts = sqlx::query_as::<_, Loadout>("SELECT * FROM loadouts")
        .fetch_all(&db)
        .await?;

    Ok(Json(loadouts))
}

pub async fn get_loadout(Path(id): Path<String>, State(db): State<PgPool>) -> Result<impl IntoResponse, super::Error> {
    let id = id.parse::<Uuid>()
        .map_err(|_| super::Error::InvalidLoadoutId)?;

    let loadout = sqlx::query_as::<_, Loadout>("SELECT FROM loadouts WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(super::Error::LoadoutNotFound { id })?;

    Ok(Json(loadout))
}

pub async fn create_loadout(State(db): State<PgPool>, Json(loadout): Json<LoadoutForCreate>) -> Result<impl IntoResponse, super::Error> {
    let created = sqlx::query_as::<_, Loadout>("INSERT INTO loadouts (merc, \"primary\", secondary, melee, name) VALUES ($1, $2, $3, $4, $5) RETURNING *")
        .bind(loadout.merc)
        .bind(loadout.primary)
        .bind(loadout.secondary)
        .bind(loadout.melee)
        .bind(loadout.name)
        .fetch_one(&db)
        .await?;

    Ok(Json(created))
}

pub async fn delete_loadout(Path(id): Path<String>, State(db): State<PgPool>) -> Result<impl IntoResponse, super::Error> {
    let id = id.parse::<Uuid>()
        .map_err(|_| super::Error::InvalidLoadoutId)?;

    let deleted_loadout = sqlx::query_as::<_, Loadout>("DELETE FROM loadouts WHERE id = $1 RETURNING *")
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(super::Error::LoadoutNotFound { id })?;

    Ok(Json(deleted_loadout))
}

pub async fn update_loadout(Path(id): Path<String>, State(db): State<PgPool>, Json(loadout): Json<LoadoutForUpdate>) -> Result<impl IntoResponse, super::Error> {
    let id = id.parse::<Uuid>()
        .map_err(|_| super::Error::InvalidLoadoutId)?;
    
    let query = r#"
        UPDATE loadouts
        SET 
            merc = $1,
            "primary" = COALESCE($2, "primary"),
            secondary = COALESCE($3, secondary),
            melee = COALESCE($4, melee),
            name = COALESCE($5, name)
        WHERE id = $6
        RETURNING *
    "#;

    let updated_loadout = sqlx::query_as::<_, Loadout>(query)
        .bind(loadout.merc)
        .bind(loadout.primary)
        .bind(loadout.secondary)
        .bind(loadout.melee)
        .bind(loadout.name)
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(super::Error::LoadoutNotFound { id })?;


    Ok(Json(updated_loadout))
}
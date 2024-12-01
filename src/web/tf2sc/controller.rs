use axum::{extract::{Path, Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Deserializer};
use sqlx::{types::Uuid, PgPool};
use strum_macros::{AsRefStr, EnumString};
use std::str::FromStr;
use serde::de;
use validator::Validate;
use super::model::{ItemSlot, Loadout, LoadoutForCreate, LoadoutForUpdate, Merc, WeaponFromView, MongoStyle};

#[derive(Deserialize)]
pub struct MercSlotParams {
    merc: Option<Merc>,
    slot: Option<ItemSlot>
}

#[derive(Deserialize)]
pub struct LoadoutParams {
    sort: Option<Sort>,
    #[serde(rename = "sortBy")]
    sort_by: Option<SortBy>
}

#[derive(Deserialize, EnumString, AsRefStr, Default)]
#[serde(rename_all = "lowercase")]
enum Sort {
    #[default]
    #[strum(serialize = "DESC")]
    Desc,
    #[strum(serialize = "ASC")]
    Asc
}

#[derive(Deserialize, EnumString, AsRefStr, Default)]
#[serde(rename_all = "lowercase")]
enum SortBy {
    #[strum(serialize = "created_at")]
    Created,
    #[default]
    #[strum(serialize = "updated_at")]
    Updated
}



// might use it in the future
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
        slot
    }  = q;

    let query = match (merc, slot) {
        (None, None) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details")
        },
        (Some(merc), None) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details WHERE merc = $1 OR merc IS NULL")
                .bind(merc)
        },
        (None, Some(slot)) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details WHERE item_slot = $1")
                .bind(slot)
        },
        (Some(merc), Some(slot)) => {
            sqlx::query_as::<_, WeaponFromView>("SELECT * FROM weapon_details WHERE (merc = $1 OR merc IS NULL) AND item_slot = $2")
                .bind(merc)
                .bind(slot)
        },
    };

    let weapons = query.fetch_all(&db).await?;
    let weapons = weapons.to_mongo_style();

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

pub async fn get_all_loadouts(State(db): State<PgPool>, Query(q): Query<LoadoutParams>) -> Result<impl IntoResponse, super::Error> {
    let LoadoutParams { 
        sort, 
        sort_by
    }  = q;

    let query = format!(
        "SELECT * FROM loadouts ORDER BY {} {}",
        sort_by.unwrap_or_default().as_ref(),
        sort.unwrap_or_default().as_ref()
    );

    let loadouts = sqlx::query_as::<_, Loadout>(&query)
        .fetch_all(&db)
        .await?;

    Ok(Json(loadouts))
}

pub async fn get_loadout(Path(id): Path<String>, State(db): State<PgPool>) -> Result<impl IntoResponse, super::Error> {
    let id = id.parse::<Uuid>()
        .map_err(|_| super::Error::InvalidLoadoutId)?;

    let loadout = sqlx::query_as::<_, Loadout>("SELECT * FROM loadouts WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(super::Error::LoadoutNotFound { id })?;

    Ok(Json(loadout))
}

pub async fn create_loadout(State(db): State<PgPool>, Json(loadout): Json<LoadoutForCreate>) -> Result<impl IntoResponse, super::Error> {
    loadout.validate()?;
    
    let created = sqlx::query_as::<_, Loadout>("INSERT INTO loadouts (merc, \"primary\", secondary, melee, name, playstyle) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *")
        .bind(loadout.merc)
        .bind(loadout.primary)
        .bind(loadout.secondary)
        .bind(loadout.melee)
        .bind(loadout.name)
        .bind(loadout.playstyle)
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

    loadout.validate()?;
    
    let query = r#"
        UPDATE loadouts
        SET 
            merc = $1,
            "primary" = COALESCE($2, "primary"),
            secondary = COALESCE($3, secondary),
            melee = COALESCE($4, melee),
            name = COALESCE($5, name),
            playstyle = COALESCE($6, playstyle)
        WHERE id = $7
        RETURNING *
    "#;

    let updated_loadout = sqlx::query_as::<_, Loadout>(query)
        .bind(loadout.merc)
        .bind(loadout.primary)
        .bind(loadout.secondary)
        .bind(loadout.melee)
        .bind(loadout.name)
        .bind(loadout.playstyle)
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(super::Error::LoadoutNotFound { id })?;

    Ok(Json(updated_loadout))
}
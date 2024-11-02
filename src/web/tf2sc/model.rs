use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::Uuid;
use sqlx::FromRow;
use sqlx::Type;
use sqlx;
use strum_macros::Display;
use strum_macros::EnumString;


#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WeaponFromView {
    id: i32,
    name: String,
    stock: bool,
    item_name: String,
    item_slot: ItemSlot,
    image_url: String,
    image_url_large: String,
    merc: Option<Merc>
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow, Deserialize)]
struct FullFlatWeapon {
    #[serde(rename = "_id")]
    id: i32,
    name: String,
    stock: bool,
    item_name: String,
    item_slot: ItemSlot,
    image_url: String,
    image_url_large: String,
    used_by_classes: Option<Vec<Merc>>,
    per_class_loadout_slots: Option<HashMap<Merc, ItemSlot>>
}

#[derive(Type, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "item_slot", rename_all = "lowercase")]
pub enum ItemSlot {
    Primary,
    Secondary,
    Melee
}

#[derive(Type, Debug, Clone, Deserialize, Serialize, EnumString, Display, PartialEq, Eq, Hash)]
#[sqlx(type_name = "merc")]
#[strum(serialize_all = "PascalCase")]
pub enum Merc {
    Scout,
    Soldier,
    Pyro,
    Demoman,
    Heavy,
    Engineer,
    Medic,
    Sniper,
    Spy
}

#[derive(Debug, Clone, FromRow, Deserialize, Serialize)]
pub struct Loadout {
    id: Uuid,
    merc: Merc,
    primary: i32,
    secondary: i32,
    melee: i32,
    name: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadoutForCreate {
    pub merc: Merc,
    pub primary: i32,
    pub secondary: i32,
    pub melee: i32,
    pub name: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadoutForUpdate {
    pub merc: Merc,
    pub primary: Option<i32>,
    pub secondary: Option<i32>,
    pub melee: Option<i32>,
    pub name: Option<String>
}


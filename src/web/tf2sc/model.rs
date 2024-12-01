use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::Uuid;
use sqlx::FromRow;
use sqlx::Type;
use sqlx;
use strum_macros::Display;
use strum_macros::EnumString;
use itertools::Itertools;
use validator::Validate;


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

// TODO: use associated type
pub trait MongoStyle {
    type Output;

    fn to_mongo_style(self) -> Self::Output;
}

impl MongoStyle for Vec<WeaponFromView> {
    type Output = Vec<MongoStyleWeapon>;

    fn to_mongo_style(self) -> Vec<MongoStyleWeapon> {
        // regular -> put merc in used_by_classes, per_class_loadout_slots is None

        // shotgun-like -> put merc in used_by_classes, (put merc: item_slot) in per_class_loadout_slots

        // prinny-like -> used_by_classes is None, per_class_loadout_slots is None

        // edge case (used by any class but each can use it in different slots (most likely impossible)) - whatever (not implemented)

        self.into_iter()
            .chunk_by(|w| w.id)
            .into_iter()
            .filter_map(|(_weapon_id, weapon_group)| {
                let mongo_weapon = weapon_group.fold(MongoStyleWeaponBuilder::default(), |weapon_builder, weapon| {
                    let slot = weapon.item_slot.clone();
                    let merc = weapon.merc.clone();

                    weapon_builder.put_if_empty(weapon).add_merc(merc, slot)
                });

                // will be None only really if the weapons passed in from above are an empty vec (should never occur)
                mongo_weapon.build()
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MongoStyleWeapon {
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

#[derive(Debug, Clone, Default)]
struct MongoStyleWeaponBuilder {
    // initialized: bool,  // might improve performance ever so slightly, because of the checks in .build() (like 1ns improvement idk)
    id: Option<i32>,
    name: Option<String>,
    stock: Option<bool>,
    item_name: Option<String>,
    item_slot: Option<ItemSlot>,
    image_url: Option<String>,
    image_url_large: Option<String>,
    used_by_classes: Option<Vec<Merc>>,
    per_class_loadout_slots: Option<HashMap<Merc, ItemSlot>>
}

impl MongoStyleWeaponBuilder {
    pub fn build(self) -> Option<MongoStyleWeapon> {
        Some(MongoStyleWeapon {
            id: self.id?,
            name: self.name?,
            stock: self.stock?,
            item_name: self.item_name?,
            item_slot: self.item_slot?,
            image_url: self.image_url?,
            image_url_large: self.image_url_large?,
            per_class_loadout_slots: if self.used_by_classes.as_ref().map(|v| v.len()).unwrap_or(0) > 1 {
                self.per_class_loadout_slots   
            } else {
                None
            },
            used_by_classes: self.used_by_classes,
        })
    }

    pub fn put(self, weapon: WeaponFromView) -> Self {
        Self {
            id: Some(weapon.id),
            name: Some(weapon.name),
            stock: Some(weapon.stock),
            item_name: Some(weapon.item_name),
            item_slot: Some(weapon.item_slot),
            image_url: Some(weapon.image_url),
            image_url_large: Some(weapon.image_url_large),
            used_by_classes: None,
            per_class_loadout_slots: None
        }
    }

    pub fn add_merc(mut self, merc: Option<Merc>, slot: ItemSlot) -> Self {
        if let Some(merc) = merc {
            self.used_by_classes.get_or_insert_with(Vec::new).push(merc.clone());
            self.per_class_loadout_slots.get_or_insert_with(HashMap::new).insert(merc, slot);
        }

        self
    }

    #[deprecated]
    #[allow(dead_code)]
    fn put_field_if_empty<T>(field: &mut Option<T>, value: T) {
        if field.is_none() {
            *field = Some(value);
        }
    }

    pub fn put_if_empty(self, weapon: WeaponFromView) -> Self {
        // hopefully this is sufficient, but this should always be triggerd on the 2nd 3rd etc passthrough
        if self.id.is_some() {
            return self;
        }

        self.put(weapon)
    }
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
    #[serde(rename(serialize = "_id"))]
    pub id: Uuid,
    pub merc: Merc,
    pub primary: i32,
    pub secondary: i32,
    pub melee: i32,
    pub name: String,
    pub playstyle: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: chrono::DateTime<chrono::Utc>
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoadoutForCreate {
    pub merc: Merc,
    pub primary: i32,
    pub secondary: i32,
    pub melee: i32,
    #[validate(length(min = 3))]
    pub name: String,
    #[validate(length(min = 3))]
    pub playstyle: String
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoadoutForUpdate {
    pub merc: Merc,
    pub primary: Option<i32>,
    pub secondary: Option<i32>,
    pub melee: Option<i32>,
    #[validate(length(min = 3))]
    pub name: Option<String>,
    #[validate(length(min = 3))]
    pub playstyle: Option<String>
}


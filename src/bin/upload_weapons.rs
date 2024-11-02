use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use serde_json::from_reader;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Type;
use sqlx;


#[derive(Debug, Clone, FromRow)]
struct Weapon {
    id: i32,
    name: String,
    stock: bool,
    item_name: String,
    item_slot: ItemSlot,
    image_url: String,
    image_url_large: String,
}

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

#[derive(Type, Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "item_slot", rename_all = "lowercase")]
pub enum ItemSlot {
    Primary,
    Secondary,
    Melee
}

#[derive(Type, Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[sqlx(type_name = "merc")]
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

// weapon_used_by_classes
#[derive(Debug, Clone, FromRow)]
pub struct WeaponUsedByClass {
    weapon_id: i32,
    merc: Merc,
}

// weapon_per_class_loadout_slots
#[derive(Debug, Clone, FromRow)]
pub struct WeaponPerClassLoadoutSlot {
    weapon_id: i32,
    merc: Merc,
    item_slot: ItemSlot,
}




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("src/bin/weapons.json")?;
    let reader = BufReader::new(f);

    let full_weapons: Vec<FullFlatWeapon> = from_reader(reader)?;

    let _ubc: Vec<WeaponUsedByClass> = full_weapons.iter()
        .flat_map(|w| {
            w.used_by_classes
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|merc| WeaponUsedByClass {
                    weapon_id: w.id,
                    merc
                })
        })
        .collect::<Vec<_>>();

    let _a: Vec<WeaponPerClassLoadoutSlot> = full_weapons.iter()
        .flat_map(|w| {
            w.per_class_loadout_slots
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|(merc, item_slot)| WeaponPerClassLoadoutSlot{
                    weapon_id: w.id,
                    merc,
                    item_slot
                })
        })
        .collect::<Vec<_>>();

    let _res: Vec<Weapon> = full_weapons.into_iter().map(|w| Weapon {
            id: w.id,
            name: w.name,
            stock: w.stock,
            item_name: w.item_name,
            item_slot: w.item_slot,
            image_url: w.image_url,
            image_url_large: w.image_url_large
        })
        .collect::<Vec<_>>();

    panic!("STOP RIGHT THERE CRIMINAL SCUM, NOBODY RE-INSERTS WEAPONS ON MY WATCH");

    let _neon_db = PgPool::connect("nuh uh").await.unwrap();
    println!("connected to neon");


    let weapon_ids: Vec<i32> = _ubc.iter().map(|w| w.weapon_id).collect();
    let mercs: Vec<Merc> = _ubc.iter().map(|w| w.merc.clone()).collect();

    let sql = r#"
        INSERT INTO weapon_used_by_classes (weapon_id, merc)
        SELECT * FROM UNNEST(
            $1::INT[],
            $2::merc[]
        )
    "#;

    sqlx::query(sql)
        .bind(weapon_ids)
        .bind(mercs)
        .execute(&_neon_db)
        .await?;
    println!("inserted ubc");

    let weapon_ids: Vec<i32> = _a.iter().map(|w| w.weapon_id).collect();
    let mercs: Vec<Merc> = _a.iter().map(|w| w.merc.clone()).collect();
    let item_slots: Vec<ItemSlot> = _a.iter().map(|w| w.item_slot.clone()).collect();

    let sql = r#"
        INSERT INTO weapon_per_class_loadout_slots (weapon_id, merc, item_slot)
        SELECT * FROM UNNEST(
            $1::INT[],
            $2::merc[],
            $3::item_slot[]
        )
    "#;

    sqlx::query(sql)
        .bind(weapon_ids)
        .bind(mercs)
        .bind(item_slots)
        .execute(&_neon_db)
        .await?;

    println!("per class loadout slots");


    let ids: Vec<i32> = _res.iter().map(|w| w.id).collect();
    let names: Vec<String> = _res.iter().map(|w| w.name.clone()).collect();
    let stocks: Vec<bool> = _res.iter().map(|w| w.stock).collect();
    let item_names: Vec<String> = _res.iter().map(|w| w.item_name.clone()).collect();
    let item_slots: Vec<ItemSlot> = _res.iter().map(|w| w.item_slot.clone()).collect();
    let image_urls: Vec<String> = _res.iter().map(|w| w.image_url.clone()).collect();
    let image_url_larges: Vec<String> = _res.iter().map(|w| w.image_url_large.clone()).collect();

    let sql = r#"
        INSERT INTO weapons (id, name, stock, item_name, item_slot, image_url, image_url_large)
        SELECT * FROM UNNEST(
            $1::INT[],
            $2::TEXT[],
            $3::BOOL[],
            $4::TEXT[],
            $5::item_slot[],
            $6::TEXT[],
            $7::TEXT[]
        )
    "#;

    sqlx::query(sql)
    .bind(ids)
    .bind(names)
    .bind(stocks)
    .bind(item_names)
    .bind(item_slots)
    .bind(image_urls)
    .bind(image_url_larges)
    .execute(&_neon_db)
    .await?;

    println!("hopefully");


    Ok(())
}
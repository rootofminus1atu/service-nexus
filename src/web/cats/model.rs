use serde::{Deserialize, Serialize};
use mongodb::bson::DateTime;
use crate::helpers::split_and_collect;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breed {
    pub id: String,
    pub name: String,
    pub temperament: Vec<String>,
    pub alt_names: Vec<String>,
    pub origin: String,
    pub country_code: String,
    pub description: String,
    pub wikipedia_url: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cat {
    #[serde(rename = "_id")]
    pub _id: String,
    pub img_url: String,
    pub breed: Breed,
    pub rarity: String,
    pub pet_name: String,
    pub full_name: String,
    pub created_at: DateTime,
    pub updated_at: DateTime
}

impl From<Cat> for CatForCreate {
    fn from(cat: Cat) -> Self {
        CatForCreate {
            _id: cat._id,
            img_url: cat.img_url,
            breed: cat.breed,
            rarity: cat.rarity,
            pet_name: cat.pet_name,
            full_name: cat.full_name,
            created_at: cat.created_at,
            updated_at: cat.updated_at
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatForCreate {
    #[serde(rename = "_id")]
    pub _id: String,
    pub img_url: String,
    pub breed: Breed,
    pub rarity: String,
    pub pet_name: String,
    pub full_name: String,
    pub created_at: DateTime,
    pub updated_at: DateTime
}

#[derive(Debug, Clone)]
pub struct CatHalfProcessed {
    pub _id: String,
    pub img_url: String,
    pub breed: Breed,
}

impl CatHalfProcessed {
    pub fn finalize_processing(&self, rarity: String, pet_name: String, full_name: String) -> CatForCreate {
        CatForCreate {
            _id: self._id.clone(),
            img_url: self.img_url.clone(),
            breed: self.breed.clone(),
            rarity,
            pet_name,
            full_name,
            created_at: DateTime::now(),
            updated_at: DateTime::now()
        }
    }
}

/// cat as a response from RandomCatAPI
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatUnprocessed {
    pub breeds: Vec<BreedUnprocessed>,
    pub id: String,
    pub url: String,
}

impl CatUnprocessed {
    // processed
    pub fn start_processing(&self) -> Result<CatHalfProcessed, super::Error> {
        let breed = self.breeds  
            .iter()
            .next()
            .ok_or(super::Error::NoBreedsFromRandomCatApi)?
            .clone()
            .process();

        Ok(CatHalfProcessed {
            _id: self.id.clone(),
            img_url: self.url.clone(),
            breed,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreedUnprocessed {
    pub id: String,
    pub name: String,
    pub temperament: String,  // its a vec as a string, csv-like
    pub origin: String,
    pub country_code: String,
    pub description: String,
    pub alt_names: String,
    pub wikipedia_url: String
}

/*
pub id: String,
    pub name: String,
    pub temperament: Vec<String>,
    pub alt_names: Vec<String>,
    pub origin: String,
    pub country_code: String,
    pub description: String,
    pub wikipedia_url: String
*/

impl BreedUnprocessed {
    pub fn process(&self) -> Breed {
        Breed {
            id: self.id.clone(),
            name: self.name.clone(),
            temperament: split_and_collect(&self.temperament, ','),
            alt_names: split_and_collect(&self.alt_names, ','),
            origin: self.origin.clone(),
            country_code: self.country_code.clone(),
            description: self.description.clone(),
            wikipedia_url: self.wikipedia_url.clone(),
        }
    }
}


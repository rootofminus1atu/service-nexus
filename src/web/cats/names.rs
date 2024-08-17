
use serde::Deserialize;
use crate::{helpers::random_choice, web::ClientWithKeys};
use super::model::Breed;


const COUNTRY_CODES: &[&'static str] = &[
    "AU", "BR", "CA", "CH", "DE", "DK", "ES", "FI", "FR", "GB", "IE", "IN", "IR", "MX", "NL", "NO", "NZ", "RS", "TR", "UA", "US"
];

pub async fn get_random_name_from_country(country_code: &str, client: ClientWithKeys) -> Result<String, super::Error> {
    let country_code = country_code.to_uppercase();

    let country_code = if COUNTRY_CODES.contains(&country_code.as_str()) {
        country_code
    }  else {
        random_choice(COUNTRY_CODES).unwrap().to_string()
    };
    
    let person = client.client
        .get(format!("https://randomuser.me/api/?nat={}", country_code))
        .send()
        .await?
        .json::<RandomPersonResponse>() 
        .await?
        .results
        .into_iter()
        .next()
        .ok_or(super::Error::NoPeopleFromRandomUserApi)?;
    
    Ok(person.name.first)
}

pub fn get_random_full_name(breed: &Breed, pet_name: &str) -> String {
    let random_temperament = random_choice(&breed.temperament)
        .cloned()
        .unwrap_or("Nice".into());

    format!("{}, the {} {}", pet_name, random_temperament, breed.name)
}

#[derive(Debug, Clone, Deserialize)]
struct RandomPersonResponse {
    results: Vec<RandomPerson>
}

#[derive(Debug, Clone, Deserialize)]
struct RandomPerson {
    _gender: String,
    name: RandomPersonName,
    _nat: String
}

#[derive(Debug, Clone, Deserialize)]
struct RandomPersonName {
    _title: String,
    first: String,
    _last: String
}
use rand::Rng;
use tracing::info;

const RARITIES: &[&'static str] = &[
    "COMMON",
    "RARE",
    "LEGENDARY",
    "MYTHIC"
];

const RARITY_FACTOR: f64 = 0.3;

/// Gets the rarity based on a random number.
fn get_rarity_for_random_num(random_num: f64) -> &'static str {
    let mut index = 0;
    let mut factor = RARITY_FACTOR;

    while random_num < factor && index < RARITIES.len() - 1 {
        index += 1;
        factor *= RARITY_FACTOR;
    }

    RARITIES[index]
}

/// Generates a random rarity name based on the predefined rarities and their probabilities.
pub fn get_random_rarity() -> &'static str {
    let rand: f64 = rand::thread_rng().gen();
    let rarity = get_rarity_for_random_num(rand);
    info!("{} -> {}", rand, rarity);
    rarity
}
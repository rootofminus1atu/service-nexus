use rand::prelude::SliceRandom;

/// Splits the input string by the given delimiter, trims each part, and collects them into a vector.
/// If the input string is empty, returns an empty vector.
/// 
/// ## Examples
///
/// ```
/// let result = split_and_collect("a, b, c", ',');
/// assert_eq!(result, vec!["a", "b", "c"]);
///
/// let result = split_and_collect("", ',');
/// assert_eq!(result, Vec::<String>::new());
/// ```
pub fn split_and_collect(input: &str, delimiter: char) -> Vec<String> {
    if input.is_empty() {
        Vec::new()
    } else {
        input.split(delimiter).map(|s| s.trim().to_owned()).collect()
    }
}



pub fn random_choice<T>(items: &[T]) -> Option<&T> {
    let mut rng = rand::thread_rng();
    items.choose(&mut rng)
}
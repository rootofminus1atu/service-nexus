use std::collections::{BTreeMap, HashMap};


pub fn merge_vecs<T: Eq + Clone>(vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> {
    let mut result = vec1.clone();

    for item in vec2 {
        if !result.contains(&item) {
            result.push(item);
        }
    }

    result
}

pub fn merge_btreemaps<K, V>(map1: BTreeMap<K, V>, map2: BTreeMap<K, V>) -> BTreeMap<K, V>
where
    K: Ord,
{
    let mut merged_map = map1;
    merged_map.extend(map2);
    merged_map
}

pub fn merge_hashmaps<K, V>(map1: HashMap<K, V>, map2: HashMap<K, V>) -> HashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    let mut merged_map = map1;
    merged_map.extend(map2);
    merged_map
}

/// Can be used for lifting a function made for merging lists (so that it could merge optional lists instead)
pub fn kind_option_lift<F, T>(f: F) -> impl Fn(Option<T>, Option<T>) -> Option<T>
where
    F: Fn(T, T) -> T,
    T: Default,
{
    move |opt1: Option<T>, opt2: Option<T>| {
        match (opt1, opt2) {
            (Some(c1), Some(c2)) => Some(f(c1, c2)),
            (Some(c1), None) => Some(f(c1, T::default())),
            (None, Some(c2)) => Some(f(T::default(), c2)),
            (None, None) => None,
        }
    }
}

/// After the lift, the function is applied only if both `Options` are `Some`. See also `kind_option_lift`
pub fn strict_option_lift<F, T>(f: F) -> impl Fn(Option<T>, Option<T>) -> Option<T> 
where
    F: Fn(T, T) -> T
{
    move |opt1: Option<T>, opt2: Option<T>| {
        match (opt1, opt2) {
            (Some(c1), Some(c2)) => Some(f(c1, c2)),
            _ => None,
        }
    }
}



#[deprecated]
pub fn merge_opbtreemaps<K, V>(map1: Option<BTreeMap<K, V>>, map2: Option<BTreeMap<K, V>>) -> Option<BTreeMap<K, V>>
where
    K: Ord
{
    match (map1, map2) {
        (Some(mut m1), Some(m2)) => {
            m1.extend(m2);
            Some(m1)
        }
        (Some(m), None) | (None, Some(m)) => Some(m),
        (None, None) => None,
    }
}

#[deprecated]
pub fn merge_opvecs<T: Eq + Clone>(opt1: Option<Vec<T>>, opt2: Option<Vec<T>>) -> Option<Vec<T>> {
    let mut result: Vec<T> = Vec::new();

    if let Some(vec1) = opt1 {
        result.extend(vec1);
    }

    if let Some(vec2) = opt2 {
        for item in vec2 {
            if !result.contains(&item) {
                result.push(item);
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
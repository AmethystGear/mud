use anyhow::{anyhow, Result};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

pub fn false_bool() -> bool {
    false
}

pub fn zero_u64() -> u64 {
    0
}

pub fn zero_i64() -> i64 {
    0
}

pub fn zero_f64() -> f64 {
    0.0
}

pub fn one_f64() -> f64 {
    1.0
}

pub fn empty_hmap<K, V>() -> HashMap<K, V> {
    HashMap::new()
}

pub fn empty_vec<T>() -> Vec<T> {
    Vec::new()
}

pub fn empty_string() -> String {
    "".to_string()
}

pub fn default_png_string() -> String {
    "default.png".to_string()
}

pub fn no_description() -> String {
    "no description".into()
}

pub fn map_key<A: From<B> + Eq + Hash + Debug, B, C>(
    val: HashMap<B, C>,
    types: &HashSet<A>,
) -> Result<HashMap<A, C>> {
    let mut map = HashMap::new();
    for (k, v) in val {
        let val = A::from(k);
        if !types.contains(&val) {
            return Err(anyhow!(format!("{:?} does not contain {:?}", types, val)));
        }
        map.insert(val, v);
    }
    return Ok(map);
}


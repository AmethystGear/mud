use anyhow::{anyhow, Result};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

pub fn default_false() -> bool {
    false
}

pub fn default_true() -> bool {
    true
}

pub fn default_u64() -> u64 {
    0
}

pub fn default_i64() -> i64 {
    0
}

pub fn default_f64() -> f64 {
    0.0
}

pub fn default_hmap<K, V>() -> HashMap<K, V> {
    HashMap::new()
}

pub fn default_string() -> String {
    "".to_string()
}

pub fn map<A: From<B> + Eq + Hash + Debug, B, C>(
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

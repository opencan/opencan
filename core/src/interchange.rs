use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

pub fn sorted_map<S: Serializer, K: Serialize + Ord, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let items: Vec<(_, _)> = value.iter().collect();
    BTreeMap::from_iter(items).serialize(serializer)
}

pub fn sorted_bimap_by_second<
    S: Serializer,
    A: Hash + Eq + Serialize + Ord,
    B: Hash + Serialize + Ord,
>(
    value: &bimap::BiMap<A, B>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut items: Vec<(_, _)> = value.iter().collect();
    items.sort_by(|a, b| a.1.cmp(b.1));

    let mut map = serializer.serialize_map(Some(items.len()))?;
    for (k, v) in items {
        map.serialize_entry(k, v)?;
    }
    map.end()
}

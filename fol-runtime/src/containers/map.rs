use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct FolMap<K, V>(BTreeMap<K, V>);

impl<K: Ord, V> FolMap<K, V> {
    pub fn new(values: BTreeMap<K, V>) -> Self {
        Self(values)
    }

    pub fn from_pairs(values: Vec<(K, V)>) -> Self {
        Self(values.into_iter().collect())
    }

    pub fn as_map(&self) -> &BTreeMap<K, V> {
        &self.0
    }

    pub fn into_map(self) -> BTreeMap<K, V> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }
}

impl<K: Ord, V> From<BTreeMap<K, V>> for FolMap<K, V> {
    fn from(values: BTreeMap<K, V>) -> Self {
        Self::new(values)
    }
}

impl<K: Ord, V> From<FolMap<K, V>> for BTreeMap<K, V> {
    fn from(values: FolMap<K, V>) -> Self {
        values.into_map()
    }
}

#[cfg(test)]
mod tests {
    use super::FolMap;
    use std::collections::BTreeMap;

    #[test]
    fn fol_map_wraps_deterministic_key_ordered_storage() {
        let values = FolMap::new(BTreeMap::from([("lin", 2), ("ada", 1)]));

        assert_eq!(values.len(), 2);
        assert!(!values.is_empty());
        assert_eq!(values.get(&"ada"), Some(&1));
        assert_eq!(
            values.as_map().keys().copied().collect::<Vec<_>>(),
            vec!["ada", "lin"]
        );
        assert_eq!(BTreeMap::from(values).get("lin"), Some(&2));
    }

    #[test]
    fn fol_map_deterministic_constructor_orders_keys_and_keeps_last_value() {
        let values = FolMap::from_pairs(vec![("lin", 2), ("ada", 1), ("lin", 4)]);

        assert_eq!(
            values.as_map().keys().copied().collect::<Vec<_>>(),
            vec!["ada", "lin"]
        );
        assert_eq!(values.get(&"lin"), Some(&4));
    }
}

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct FolSet<T>(BTreeSet<T>);

impl<T: Ord> FolSet<T> {
    pub fn new(values: BTreeSet<T>) -> Self {
        Self(values)
    }

    pub fn as_set(&self) -> &BTreeSet<T> {
        &self.0
    }

    pub fn into_set(self) -> BTreeSet<T> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.0.contains(value)
    }
}

impl<T: Ord> From<BTreeSet<T>> for FolSet<T> {
    fn from(values: BTreeSet<T>) -> Self {
        Self::new(values)
    }
}

impl<T: Ord> From<FolSet<T>> for BTreeSet<T> {
    fn from(values: FolSet<T>) -> Self {
        values.into_set()
    }
}

#[cfg(test)]
mod tests {
    use super::FolSet;
    use std::collections::BTreeSet;

    #[test]
    fn fol_set_wraps_deterministic_ordered_storage() {
        let values = FolSet::new(BTreeSet::from([3, 1, 2]));

        assert_eq!(values.len(), 3);
        assert!(values.contains(&2));
        assert_eq!(
            values.as_set().iter().copied().collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            BTreeSet::from(values).into_iter().collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }
}

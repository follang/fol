#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct FolSeq<T>(Vec<T>);

impl<T> FolSeq<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self(values)
    }

    pub fn from_items(values: Vec<T>) -> Self {
        Self(values)
    }

    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> From<Vec<T>> for FolSeq<T> {
    fn from(values: Vec<T>) -> Self {
        Self::new(values)
    }
}

impl<T> From<FolSeq<T>> for Vec<T> {
    fn from(values: FolSeq<T>) -> Self {
        values.into_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::FolSeq;

    #[test]
    fn fol_seq_wraps_owned_sequence_storage() {
        let values = FolSeq::new(vec![1, 2, 3]);

        assert_eq!(values.as_slice(), &[1, 2, 3]);
        assert_eq!(values.len(), 3);
        assert!(!values.is_empty());
        assert_eq!(Vec::from(values), vec![1, 2, 3]);
    }

    #[test]
    fn fol_seq_deterministic_constructor_keeps_input_order() {
        let values = FolSeq::from_items(vec![3, 1, 2]);

        assert_eq!(values.as_slice(), &[3, 1, 2]);
    }
}

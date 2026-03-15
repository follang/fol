#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct FolVec<T>(Vec<T>);

impl<T> FolVec<T> {
    pub fn new(values: Vec<T>) -> Self {
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

impl<T> From<Vec<T>> for FolVec<T> {
    fn from(values: Vec<T>) -> Self {
        Self::new(values)
    }
}

impl<T> From<FolVec<T>> for Vec<T> {
    fn from(values: FolVec<T>) -> Self {
        values.into_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::FolVec;

    #[test]
    fn fol_vec_wraps_owned_vector_storage() {
        let values = FolVec::new(vec![1, 2, 3]);

        assert_eq!(values.as_slice(), &[1, 2, 3]);
        assert_eq!(values.len(), 3);
        assert!(!values.is_empty());
        assert_eq!(Vec::from(values), vec![1, 2, 3]);
    }
}

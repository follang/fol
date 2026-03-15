//! Runtime-owned builtin and intrinsic hook support.

use crate::{
    containers::{FolArray, FolMap, FolSeq, FolSet, FolVec},
    strings::FolStr,
    value::FolInt,
};

pub trait FolLength {
    fn fol_length(&self) -> FolInt;
}

impl FolLength for FolStr {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T, const N: usize> FolLength for FolArray<T, N> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T> FolLength for FolVec<T> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T> FolLength for FolSeq<T> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<T: Ord> FolLength for FolSet<T> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

impl<K: Ord, V> FolLength for FolMap<K, V> {
    fn fol_length(&self) -> FolInt {
        self.len() as FolInt
    }
}

pub fn module_name() -> &'static str {
    "builtins"
}

#[cfg(test)]
mod tests {
    use super::FolLength;
    use crate::{
        containers::{FolArray, FolMap, FolSeq, FolSet, FolVec},
        strings::FolStr,
    };
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn runtime_length_trait_covers_current_v1_families() {
        let text = FolStr::from("Ada");
        let array: FolArray<i64, 3> = [1, 2, 3];
        let vector = FolVec::from_items(vec![1, 2]);
        let sequence = FolSeq::from_items(vec![1, 2, 3, 4]);
        let set = FolSet::new(BTreeSet::from([1, 2, 3]));
        let map = FolMap::new(BTreeMap::from([("ada", 1), ("lin", 2)]));

        assert_eq!(text.fol_length(), 3);
        assert_eq!(array.fol_length(), 3);
        assert_eq!(vector.fol_length(), 2);
        assert_eq!(sequence.fol_length(), 4);
        assert_eq!(set.fol_length(), 3);
        assert_eq!(map.fol_length(), 2);
    }
}

//! Runtime container families used by executable FOL V1 programs.

mod vector;
mod sequence;
mod set;
mod map;

pub use map::FolMap;
pub use set::FolSet;
pub use sequence::FolSeq;
pub use vector::FolVec;

/// Fixed-size array strategy for FOL `arr[...]`.
///
/// Arrays stay native Rust arrays in the runtime layer so generated backends can
/// rely on stable fixed-size layout without an extra wrapper type.
pub type FolArray<T, const N: usize> = [T; N];

pub fn module_name() -> &'static str {
    "containers"
}

#[cfg(test)]
mod tests {
    use super::{FolArray, FolMap, FolSeq, FolSet, FolVec};

    #[test]
    fn fol_array_keeps_native_fixed_size_behavior() {
        let values: FolArray<i64, 3> = [1, 2, 3];

        assert_eq!(values.len(), 3);
        assert_eq!(values[0], 1);
        assert_eq!(values[2], 3);
        assert_eq!(values.iter().copied().sum::<i64>(), 6);
    }

    #[test]
    fn vector_module_exports_runtime_vector_type() {
        let values = FolVec::new(vec![1, 2, 3]);

        assert_eq!(values.len(), 3);
    }

    #[test]
    fn sequence_module_exports_runtime_sequence_type() {
        let values = FolSeq::new(vec![1, 2, 3]);

        assert_eq!(values.len(), 3);
    }

    #[test]
    fn set_module_exports_runtime_set_type() {
        let values = FolSet::new(std::collections::BTreeSet::from([1, 2, 3]));

        assert_eq!(values.len(), 3);
    }

    #[test]
    fn map_module_exports_runtime_map_type() {
        let values = FolMap::new(std::collections::BTreeMap::from([
            ("ada", 1),
            ("lin", 2),
        ]));

        assert_eq!(values.len(), 2);
    }

    #[test]
    fn deterministic_container_constructors_are_available_from_public_types() {
        let vec = FolVec::from_items(vec![1, 2]);
        let seq = FolSeq::from_items(vec![1, 2]);
        let set = FolSet::from_items(vec![2, 1, 2]);
        let map = FolMap::from_pairs(vec![("lin", 2), ("ada", 1)]);

        assert_eq!(vec.as_slice(), &[1, 2]);
        assert_eq!(seq.as_slice(), &[1, 2]);
        assert_eq!(set.len(), 2);
        assert_eq!(map.len(), 2);
    }
}

//! Runtime container families used by executable FOL V1 programs.

mod vector;
mod sequence;
mod set;
mod map;

use crate::{
    error::{RuntimeError, RuntimeErrorKind},
    value::FolInt,
};
use std::fmt::Display;

pub use map::FolMap;
pub use set::FolSet;
pub use sequence::FolSeq;
pub use vector::FolVec;

/// Fixed-size array strategy for FOL `arr[...]`.
///
/// Arrays stay native Rust arrays in the runtime layer so generated backends can
/// rely on stable fixed-size layout without an extra wrapper type.
pub type FolArray<T, const N: usize> = [T; N];

fn normalize_index(index: FolInt, len: usize) -> Result<usize, RuntimeError> {
    if index < 0 {
        return Err(RuntimeError::new(
            RuntimeErrorKind::InvalidInput,
            format!("index out of bounds: the len is {len} but the index is {index}"),
        ));
    }

    let index = index as usize;
    if index >= len {
        return Err(RuntimeError::new(
            RuntimeErrorKind::InvalidInput,
            format!("index out of bounds: the len is {len} but the index is {index}"),
        ));
    }

    Ok(index)
}

pub fn index_array<T, const N: usize>(
    values: &FolArray<T, N>,
    index: FolInt,
) -> Result<&T, RuntimeError> {
    let index = normalize_index(index, values.len())?;
    Ok(&values[index])
}

pub fn index_vec<T>(values: &FolVec<T>, index: FolInt) -> Result<&T, RuntimeError> {
    let index = normalize_index(index, values.len())?;
    Ok(&values.as_slice()[index])
}

pub fn index_seq<T>(values: &FolSeq<T>, index: FolInt) -> Result<&T, RuntimeError> {
    let index = normalize_index(index, values.len())?;
    Ok(&values.as_slice()[index])
}

pub fn lookup_map<'a, K: Ord, V>(
    values: &'a FolMap<K, V>,
    key: &K,
) -> Result<&'a V, RuntimeError> {
    values
        .get(key)
        .ok_or_else(|| RuntimeError::new(RuntimeErrorKind::InvalidInput, "missing map key"))
}

fn join_rendered<I>(items: I) -> String
where
    I: IntoIterator<Item = String>,
{
    items.into_iter().collect::<Vec<_>>().join(", ")
}

pub fn render_array<T: Display, const N: usize>(values: &FolArray<T, N>) -> String {
    format!("arr[{}]", join_rendered(values.iter().map(|value| value.to_string())))
}

pub fn render_vec<T: Display>(values: &FolVec<T>) -> String {
    format!(
        "vec[{}]",
        join_rendered(values.as_slice().iter().map(|value| value.to_string()))
    )
}

pub fn render_seq<T: Display>(values: &FolSeq<T>) -> String {
    format!(
        "seq[{}]",
        join_rendered(values.as_slice().iter().map(|value| value.to_string()))
    )
}

pub fn render_set<T: Display + Ord>(values: &FolSet<T>) -> String {
    format!(
        "set{{{}}}",
        join_rendered(values.as_set().iter().map(|value| value.to_string()))
    )
}

pub fn render_map<K: Display + Ord, V: Display>(values: &FolMap<K, V>) -> String {
    format!(
        "map{{{}}}",
        join_rendered(
            values
                .as_map()
                .iter()
                .map(|(key, value)| format!("{key}: {value}"))
        )
    )
}

pub fn module_name() -> &'static str {
    "containers"
}

#[cfg(test)]
mod tests {
    use super::{
        index_array, index_seq, index_vec, lookup_map, render_array, render_map, render_seq,
        render_set, render_vec, FolArray, FolMap, FolSeq, FolSet, FolVec,
    };
    use crate::error::RuntimeErrorKind;

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

    #[test]
    fn runtime_index_helpers_cover_linear_and_map_families() {
        let array: FolArray<i64, 3> = [10, 20, 30];
        let vector = FolVec::from_items(vec![10, 20, 30]);
        let sequence = FolSeq::from_items(vec![10, 20, 30]);
        let map = FolMap::from_pairs(vec![("ada", 1), ("lin", 2)]);

        assert_eq!(index_array(&array, 1), Ok(&20));
        assert_eq!(index_vec(&vector, 2), Ok(&30));
        assert_eq!(index_seq(&sequence, 0), Ok(&10));
        assert_eq!(lookup_map(&map, &"lin"), Ok(&2));

        let failure = index_vec(&vector, -1).expect_err("negative index should fail");
        assert_eq!(failure.kind(), RuntimeErrorKind::InvalidInput);
        assert_eq!(failure.message(), "index out of bounds: the len is 3 but the index is -1");
    }

    #[test]
    fn container_render_helpers_cover_all_current_v1_families() {
        let array: FolArray<i64, 3> = [1, 2, 3];
        let vector = FolVec::from_items(vec![1, 2, 3]);
        let sequence = FolSeq::from_items(vec![1, 2, 3]);
        let set = FolSet::from_items(vec![3, 1, 2]);
        let map = FolMap::from_pairs(vec![("lin", 2), ("ada", 1)]);

        assert_eq!(render_array(&array), "arr[1, 2, 3]");
        assert_eq!(render_vec(&vector), "vec[1, 2, 3]");
        assert_eq!(render_seq(&sequence), "seq[1, 2, 3]");
        assert_eq!(render_set(&set), "set{1, 2, 3}");
        assert_eq!(render_map(&map), "map{ada: 1, lin: 2}");
    }
}

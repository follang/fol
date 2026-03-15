//! Runtime container families used by executable FOL V1 programs.

mod vector;
mod sequence;

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
    use super::{FolArray, FolSeq, FolVec};

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
}

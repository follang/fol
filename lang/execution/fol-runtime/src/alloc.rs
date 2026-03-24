//! Heap-enabled, OS-free runtime tier surface.

use crate::core::{self, RuntimeTier};
use std::{
    borrow::Borrow,
    fmt,
    ops::{Add, Deref},
};

pub const HAS_HEAP: bool = true;
pub const HAS_OS: bool = false;
pub const TIER: RuntimeTier = RuntimeTier::new("alloc", HAS_HEAP, HAS_OS);

pub fn module_name() -> &'static str {
    "alloc"
}

pub fn tier_name() -> &'static str {
    TIER.name
}

pub fn base_tier() -> RuntimeTier {
    core::capabilities()
}

pub fn capabilities() -> RuntimeTier {
    TIER
}

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FolStr(String);

impl FolStr {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for FolStr {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for FolStr {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<FolStr> for String {
    fn from(value: FolStr) -> Self {
        value.0
    }
}

impl AsRef<str> for FolStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for FolStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for FolStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Add for FolStr {
    type Output = FolStr;

    fn add(self, rhs: FolStr) -> FolStr {
        FolStr(self.0 + &rhs.0)
    }
}

impl fmt::Display for FolStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for FolStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct FolVec<T>(Vec<T>);

impl<T> FolVec<T> {
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
    use super::*;

    #[test]
    fn alloc_tier_marks_heap_without_os() {
        assert_eq!(module_name(), "alloc");
        assert_eq!(tier_name(), "alloc");
        assert!(HAS_HEAP);
        assert!(!HAS_OS);
        assert_eq!(capabilities(), TIER);
    }

    #[test]
    fn alloc_tier_builds_on_core_tier() {
        assert_eq!(base_tier(), core::TIER);
        assert!(!base_tier().has_heap);
        assert!(capabilities().has_heap);
        assert_eq!(capabilities().has_os, base_tier().has_os);
    }

    #[test]
    fn fol_str_supports_literal_and_owned_conversions() {
        let borrowed = FolStr::from("Ada");
        let owned = FolStr::new(String::from("Lin"));

        assert_eq!(borrowed.as_str(), "Ada");
        assert_eq!(owned.as_str(), "Lin");
        assert_eq!(String::from(borrowed.clone()), "Ada");
        assert_eq!(borrowed.len(), 3);
        assert!(!owned.is_empty());
    }

    #[test]
    fn fol_str_freezes_equality_order_display_and_debug_behavior() {
        let ada = FolStr::from("Ada");
        let lin = FolStr::from("Lin");

        assert_eq!(ada, FolStr::from("Ada"));
        assert!(ada < lin);
        assert_eq!(format!("{ada}"), "Ada");
        assert_eq!(format!("{ada:?}"), "\"Ada\"");
    }

    #[test]
    fn fol_str_add_concatenates_strings() {
        let hello = FolStr::from("hello ");
        let world = FolStr::from("world");

        assert_eq!((hello + world).as_str(), "hello world");
    }

    #[test]
    fn fol_vec_wraps_owned_vector_storage() {
        let values = FolVec::new(vec![1, 2, 3]);

        assert_eq!(values.as_slice(), &[1, 2, 3]);
        assert_eq!(values.len(), 3);
        assert!(!values.is_empty());
        assert_eq!(Vec::from(values), vec![1, 2, 3]);
    }

    #[test]
    fn fol_vec_deterministic_constructor_keeps_input_order() {
        let values = FolVec::from_items(vec![3, 1, 2]);

        assert_eq!(values.as_slice(), &[3, 1, 2]);
    }

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

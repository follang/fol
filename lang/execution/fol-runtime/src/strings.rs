//! Runtime string support.

use std::{borrow::Borrow, fmt, ops::{Add, Deref}};

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

pub fn module_name() -> &'static str {
    "strings"
}

#[cfg(test)]
mod tests {
    use super::FolStr;

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
}

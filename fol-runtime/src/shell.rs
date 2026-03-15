//! Shell value helpers for optional and error-like runtime wrappers.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FolOption<T> {
    Some(T),
    Nil,
}

impl<T> FolOption<T> {
    pub fn some(value: T) -> Self {
        Self::Some(value)
    }

    pub fn nil() -> Self {
        Self::Nil
    }

    pub fn is_some(&self) -> bool {
        matches!(self, Self::Some(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn as_ref(&self) -> FolOption<&T> {
        match self {
            Self::Some(value) => FolOption::Some(value),
            Self::Nil => FolOption::Nil,
        }
    }

    pub fn into_option(self) -> Option<T> {
        match self {
            Self::Some(value) => Some(value),
            Self::Nil => None,
        }
    }
}

impl<T> From<Option<T>> for FolOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(item) => Self::Some(item),
            None => Self::Nil,
        }
    }
}

impl<T> From<FolOption<T>> for Option<T> {
    fn from(value: FolOption<T>) -> Self {
        value.into_option()
    }
}

pub fn module_name() -> &'static str {
    "shell"
}

#[cfg(test)]
mod tests {
    use super::FolOption;

    #[test]
    fn fol_option_freezes_some_nil_shape_and_queries() {
        let some = FolOption::some(7);
        let nil = FolOption::<i64>::nil();

        assert!(some.is_some());
        assert!(!some.is_nil());
        assert_eq!(some.as_ref(), FolOption::Some(&7));

        assert!(nil.is_nil());
        assert!(!nil.is_some());
        assert_eq!(nil.as_ref(), FolOption::Nil);
    }

    #[test]
    fn fol_option_converts_to_and_from_rust_option() {
        let some = FolOption::from(Some("Ada"));
        let nil = FolOption::<&str>::from(None);

        assert_eq!(Option::from(some), Some("Ada"));
        assert_eq!(Option::from(nil), None);
    }
}

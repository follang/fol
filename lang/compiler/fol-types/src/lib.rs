// FOL Types - Shared types, traits, and core abstractions

#[macro_use]
pub mod r#mod;

// Basic types
pub use r#mod::*;

pub fn canonical_identifier_key(name: &str) -> String {
    name.chars()
        .filter(|ch| *ch != '_')
        .map(|ch| {
            if ch.is_ascii() {
                ch.to_ascii_lowercase()
            } else {
                ch
            }
        })
        .collect()
}

// Placeholder error trait for now
pub trait Glitch: std::error::Error + Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn Glitch>;
    fn as_any(&self) -> &dyn std::any::Any;
}

// Basic implementations will be added later
#[derive(Debug, Clone)]
pub struct BasicError {
    pub message: String,
}

impl std::fmt::Display for BasicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BasicError {}
impl Glitch for BasicError {
    fn clone_box(&self) -> Box<dyn Glitch> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Clone for Box<dyn Glitch> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_identifier_key;

    #[test]
    fn canonical_identifier_key_normalizes_ascii_case_and_underscores() {
        assert_eq!(canonical_identifier_key("Foo_Bar"), "foobar");
        assert_eq!(canonical_identifier_key("foo__bar"), "foobar");
        assert_eq!(canonical_identifier_key("MIXED_Case_Name"), "mixedcasename");
    }

    #[test]
    fn canonical_identifier_key_preserves_non_ascii_while_normalizing_ascii() {
        assert_eq!(canonical_identifier_key("Straße_Name"), "straßename");
        assert_eq!(canonical_identifier_key("Δelta_Name"), "Δeltaname");
    }
}

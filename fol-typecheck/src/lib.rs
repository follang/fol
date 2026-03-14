//! Whole-program type checking for the `V1` FOL language subset.
//!
//! This crate is introduced in stages. The early foundation slices only provide
//! the workspace boundary and a small public API surface so later commits can
//! grow semantic types, typed results, and diagnostics incrementally.

#[derive(Debug, Default)]
pub struct Typechecker;

impl Typechecker {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::Typechecker;

    #[test]
    fn typechecker_foundation_can_be_constructed() {
        let _ = Typechecker::new();
    }
}

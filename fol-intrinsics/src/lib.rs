//! Shared intrinsic registry foundations for the FOL compiler.

pub const CRATE_NAME: &str = "fol-intrinsics";

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_matches_expected_foundation_identity() {
        assert_eq!(crate_name(), "fol-intrinsics");
    }
}

//! Runtime support foundations for executable FOL V1 programs.

pub const CRATE_NAME: &str = "fol-runtime";

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_matches_expected_runtime_identity() {
        assert_eq!(crate_name(), "fol-runtime");
    }
}

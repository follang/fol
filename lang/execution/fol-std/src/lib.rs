pub fn crate_name() -> &'static str {
    "fol-std"
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_is_stable() {
        assert_eq!(super::crate_name(), "fol-std");
    }
}

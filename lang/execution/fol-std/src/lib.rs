pub fn crate_name() -> &'static str {
    "fol-std"
}

pub fn parent_runtimes() -> (&'static str, &'static str) {
    (fol_core::crate_name(), fol_alloc::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_is_stable() {
        assert_eq!(super::crate_name(), "fol-std");
    }

    #[test]
    fn std_depends_on_core_and_alloc() {
        assert_eq!(super::parent_runtimes(), ("fol-core", "fol-alloc"));
    }
}

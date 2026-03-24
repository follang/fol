pub fn crate_name() -> &'static str {
    "fol-alloc"
}

pub fn parent_runtime() -> &'static str {
    fol_core::crate_name()
}

#[cfg(test)]
mod tests {
    #[test]
    fn crate_name_is_stable() {
        assert_eq!(super::crate_name(), "fol-alloc");
    }

    #[test]
    fn alloc_depends_on_core() {
        assert_eq!(super::parent_runtime(), "fol-core");
    }
}

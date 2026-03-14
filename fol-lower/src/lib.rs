//! Lowering from typed `V1` FOL workspaces into a backend-oriented IR.

pub fn crate_name() -> &'static str {
    "fol-lower"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn lowering_crate_is_wired_into_the_workspace() {
        assert_eq!(crate_name(), "fol-lower");
    }
}

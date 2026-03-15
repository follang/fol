//! Backend foundations for turning lowered `V1` FOL workspaces into runnable artifacts.

pub const CRATE_NAME: &str = "fol-backend";

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

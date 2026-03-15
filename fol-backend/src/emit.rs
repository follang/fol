use crate::{plan_generated_crate_layout, BackendSession, EmittedRustFile};
use std::path::{Path, PathBuf};

pub fn emit_cargo_toml(session: &BackendSession) -> EmittedRustFile {
    let layout = plan_generated_crate_layout(session);
    let package_name = session.workspace_identity().crate_dir_name.clone();
    let runtime_path = runtime_dependency_path();

    EmittedRustFile {
        path: layout.cargo_toml_path,
        module_name: "cargo".to_string(),
        contents: format!(
            "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nfol-runtime = {{ path = \"{}\" }}\n",
            runtime_path.display()
        ),
    }
}

pub fn emit_main_rs(session: &BackendSession) -> EmittedRustFile {
    let layout = plan_generated_crate_layout(session);
    let entry_name = &session.entry_identity().display_name;
    let entry_candidates = session
        .entry_candidates()
        .iter()
        .map(|candidate| format!("\"{}\"", candidate.name))
        .collect::<Vec<_>>()
        .join(", ");

    EmittedRustFile {
        path: layout.main_rs_path,
        module_name: "main".to_string(),
        contents: format!(
            "use fol_runtime::prelude as rt;\n\nmod packages;\n\nfn main() {{\n    let _runtime = rt::crate_name();\n    let _entry_package = \"{entry_name}\";\n    let _entry_candidates = [{entry_candidates}];\n    let _ = (&_runtime, &_entry_package, &_entry_candidates);\n}}\n"
        ),
    }
}

fn runtime_dependency_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("fol-runtime")
}

#[cfg(test)]
mod tests {
    use super::{emit_cargo_toml, emit_main_rs};
    use crate::{testing::sample_lowered_workspace, BackendSession};

    #[test]
    fn cargo_toml_emission_keeps_runtime_dependency_and_generated_crate_identity() {
        let session = BackendSession::new(sample_lowered_workspace());

        let emitted = emit_cargo_toml(&session);

        assert_eq!(emitted.path, "Cargo.toml");
        assert_eq!(emitted.module_name, "cargo");
        assert!(emitted.contents.contains("[package]"));
        assert!(emitted.contents.contains("edition = \"2021\""));
        assert!(emitted
            .contents
            .contains(&format!("name = \"{}\"", session.workspace_identity().crate_dir_name)));
        assert!(emitted.contents.contains("[dependencies]"));
        assert!(emitted.contents.contains("fol-runtime = { path = "));
        assert!(emitted.contents.contains("/fol-runtime"));
    }

    #[test]
    fn main_rs_emission_keeps_runtime_import_and_entry_metadata_shell() {
        let session = BackendSession::new(sample_lowered_workspace());

        let emitted = emit_main_rs(&session);

        assert_eq!(emitted.path, "src/main.rs");
        assert_eq!(emitted.module_name, "main");
        assert!(emitted.contents.contains("use fol_runtime::prelude as rt;"));
        assert!(emitted.contents.contains("mod packages;"));
        assert!(emitted.contents.contains("let _entry_package = \"app\";"));
        assert!(emitted.contents.contains("\"main\""));
    }
}

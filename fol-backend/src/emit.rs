use crate::{
    mangle_package_module_name, plan_generated_crate_layout, plan_namespace_layouts,
    plan_package_layouts, BackendSession, EmittedRustFile,
};
use std::collections::BTreeMap;
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

pub fn emit_package_module_shells(session: &BackendSession) -> Vec<EmittedRustFile> {
    let package_plans = plan_package_layouts(session);
    let namespace_plans = plan_namespace_layouts(session);
    let mut namespace_modules_by_package: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for namespace_plan in namespace_plans {
        namespace_modules_by_package
            .entry(mangle_package_module_name(&namespace_plan.package_identity))
            .or_default()
            .push(namespace_plan.module_name);
    }

    let mut files = vec![EmittedRustFile {
        path: "src/packages/mod.rs".to_string(),
        module_name: "packages".to_string(),
        contents: package_plans
            .iter()
            .map(|plan| format!("pub mod {};", plan.module_name))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    }];

    for package_plan in package_plans {
        let mut namespace_modules = namespace_modules_by_package
            .remove(&package_plan.module_name)
            .unwrap_or_default();
        namespace_modules.sort();
        namespace_modules.dedup();

        files.push(EmittedRustFile {
            path: format!("{}/mod.rs", package_plan.relative_dir),
            module_name: package_plan.module_name.clone(),
            contents: namespace_modules
                .iter()
                .map(|module_name| format!("pub mod {module_name};"))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        });
    }

    files
}

fn runtime_dependency_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("fol-runtime")
}

#[cfg(test)]
mod tests {
    use super::{emit_cargo_toml, emit_main_rs, emit_package_module_shells};
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

    #[test]
    fn package_module_shell_emission_keeps_package_and_namespace_module_tree() {
        let session = BackendSession::new(sample_lowered_workspace());

        let emitted = emit_package_module_shells(&session);

        assert_eq!(emitted.len(), 3);
        assert_eq!(emitted[0].path, "src/packages/mod.rs");
        assert!(emitted[0].contents.contains("pub mod pkg__entry__app;"));
        assert!(emitted[0].contents.contains("pub mod pkg__local__shared;"));
        assert_eq!(emitted[1].path, "src/packages/pkg__entry__app/mod.rs");
        assert!(emitted[1].contents.contains("pub mod root;"));
        assert!(emitted[1].contents.contains("pub mod math;"));
        assert_eq!(emitted[2].path, "src/packages/pkg__local__shared/mod.rs");
        assert!(emitted[2].contents.contains("pub mod root;"));
        assert!(emitted[2].contents.contains("pub mod util;"));
    }
}

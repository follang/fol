use crate::{
    mangle_package_module_name, mangle_routine_name, plan_generated_crate_layout,
    plan_namespace_layouts, plan_package_layouts, render_entry_definition, render_entry_trait_impl,
    render_global_declaration, render_record_definition, render_record_trait_impl,
    render_routine_definition, render_routine_shell, BackendArtifact, BackendConfig, BackendError,
    BackendErrorKind, BackendMode, BackendResult, BackendSession, EmittedRustFile,
};
use fol_lower::LoweredType;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn emit_cargo_toml(session: &BackendSession) -> EmittedRustFile {
    let layout = plan_generated_crate_layout(session);
    let package_name = session.workspace_identity().crate_dir_name.clone();
    let runtime_path = runtime_dependency_path();

    EmittedRustFile {
        path: layout.cargo_toml_path,
        module_name: "cargo".to_string(),
        contents: format!(
            "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\nfol-runtime = {{ path = \"{}\" }}\n",
            runtime_path.display()
        ),
    }
}

pub fn emit_main_rs(session: &BackendSession) -> BackendResult<EmittedRustFile> {
    let layout = plan_generated_crate_layout(session);
    let entry_candidate = session.select_buildable_entry_candidate()?;
    let entry_name = &session.entry_identity().display_name;
    let entry_wrapper = match resolve_entry_callable(session, entry_candidate) {
        Some(EntryCallable {
            rust_path,
            recoverable: false,
        }) => format!("    let _ = {rust_path}();"),
        Some(EntryCallable {
            rust_path,
            recoverable: true,
        }) => format!(
            "    let __fol_outcome = rt::outcome_from_recoverable({rust_path}());\n    if let Some(__fol_message) = rt::printable_outcome_message(&__fol_outcome) {{\n        eprintln!(\"{{}}\", __fol_message);\n    }}\n    std::process::exit(__fol_outcome.exit_code());"
        ),
        None => "    let _entry_name = \"placeholder\";".to_string(),
    };

    Ok(EmittedRustFile {
        path: layout.main_rs_path,
        module_name: "main".to_string(),
        contents: format!(
            "use fol_runtime::prelude as rt;\n\nmod packages;\n\nfn main() {{\n    let _runtime = rt::crate_name();\n    let _entry_package = \"{entry_name}\";\n    let _entry_name = \"{}\";\n    let _ = (&_runtime, &_entry_package, &_entry_name);\n{entry_wrapper}\n}}\n",
            entry_candidate.name
        ),
    })
}

pub fn emit_package_module_shells(session: &BackendSession) -> Vec<EmittedRustFile> {
    let package_plans = plan_package_layouts(session);
    let namespace_plans = plan_namespace_layouts(session);
    let mut direct_modules_by_path: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for namespace_plan in &namespace_plans {
        let package_module = mangle_package_module_name(&namespace_plan.package_identity);
        let relative_parts = namespace_plan
            .relative_file
            .split('/')
            .map(str::to_string)
            .collect::<Vec<_>>();
        if relative_parts.is_empty() {
            continue;
        }
        let root_child = module_name_from_relative_part(&relative_parts[0]);
        direct_modules_by_path
            .entry(format!("src/packages/{package_module}/mod.rs"))
            .or_default()
            .push(root_child);

        if relative_parts.len() <= 1 {
            continue;
        }

        for index in 0..(relative_parts.len() - 1) {
            let parent_dir = if index == 0 {
                format!("src/packages/{package_module}/{}", relative_parts[0])
            } else {
                format!(
                    "src/packages/{package_module}/{}",
                    relative_parts[..=index].join("/")
                )
            };
            let child = module_name_from_relative_part(&relative_parts[index + 1]);
            direct_modules_by_path
                .entry(format!("{parent_dir}/mod.rs"))
                .or_default()
                .push(child);
        }
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
        let mut namespace_modules = direct_modules_by_path
            .remove(&format!("{}/mod.rs", package_plan.relative_dir))
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

    let mut nested_mod_paths = direct_modules_by_path.into_iter().collect::<Vec<_>>();
    nested_mod_paths.sort_by(|left, right| left.0.cmp(&right.0));
    for (path, mut module_names) in nested_mod_paths {
        module_names.sort();
        module_names.dedup();
        files.push(EmittedRustFile {
            module_name: "mod".to_string(),
            path,
            contents: module_names
                .iter()
                .map(|module_name| format!("pub mod {module_name};"))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        });
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));

    files
}

pub fn emit_namespace_module_shells(session: &BackendSession) -> Vec<EmittedRustFile> {
    plan_namespace_layouts(session)
        .into_iter()
        .map(|namespace_plan| {
            let emitted_items = render_namespace_items(session, &namespace_plan);
            let mut contents = format!(
                "use fol_runtime::prelude as rt;\n\npub(crate) const NAMESPACE_NAME: &str = \"{}\";\npub(crate) const SOURCE_UNIT_IDS: &[usize] = &[{}];\n\npub(crate) fn namespace_runtime_marker() -> &'static str {{\n    let _ = rt::crate_name();\n    NAMESPACE_NAME\n}}\n",
                namespace_plan.full_namespace,
                namespace_plan
                    .source_unit_ids
                    .iter()
                    .map(|source_unit_id| source_unit_id.0.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if !emitted_items.is_empty() {
                contents.push('\n');
                contents.push_str(&emitted_items);
            }
            EmittedRustFile {
                path: format!(
                    "src/packages/{}/{}",
                    mangle_package_module_name(&namespace_plan.package_identity),
                    namespace_plan.relative_file
                ),
                module_name: namespace_plan.module_name.clone(),
                contents,
            }
        })
        .collect()
}

fn module_name_from_relative_part(relative_part: &str) -> String {
    relative_part
        .strip_suffix(".rs")
        .map(str::to_string)
        .unwrap_or_else(|| relative_part.to_string())
}

pub fn emit_generated_crate_skeleton(session: &BackendSession) -> BackendResult<BackendArtifact> {
    let layout = plan_generated_crate_layout(session);
    let mut files = Vec::new();
    files.push(emit_cargo_toml(session));
    files.push(emit_main_rs(session)?);
    files.extend(emit_package_module_shells(session));
    files.extend(emit_namespace_module_shells(session));
    files.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(BackendArtifact::RustSourceCrate {
        root: layout.crate_dir_name,
        files,
    })
}

pub fn write_generated_crate(output_root: &Path, artifact: &BackendArtifact) -> BackendResult<PathBuf> {
    let BackendArtifact::RustSourceCrate { root, files } = artifact else {
        return Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            "write_generated_crate expects a RustSourceCrate artifact",
        ));
    };

    let crate_root = output_root.join(root);
    if crate_root.exists() {
        fs::remove_dir_all(&crate_root).map_err(|error| {
            BackendError::new(
                BackendErrorKind::EmissionFailure,
                format!("failed to clean generated crate root '{}': {error}", crate_root.display()),
            )
        })?;
    }
    fs::create_dir_all(&crate_root).map_err(|error| {
        BackendError::new(
            BackendErrorKind::EmissionFailure,
            format!("failed to create generated crate root '{}': {error}", crate_root.display()),
        )
    })?;

    for file in files {
        let path = crate_root.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                BackendError::new(
                    BackendErrorKind::EmissionFailure,
                    format!("failed to create generated module dir '{}': {error}", parent.display()),
                )
            })?;
        }
        fs::write(&path, &file.contents).map_err(|error| {
            BackendError::new(
                BackendErrorKind::EmissionFailure,
                format!("failed to write generated file '{}': {error}", path.display()),
            )
        })?;
    }

    Ok(crate_root)
}

pub fn prepare_generated_build_dir(output_root: &Path) -> BackendResult<PathBuf> {
    let build_root = output_root.join("fol-backend");
    fs::create_dir_all(&build_root).map_err(|error| {
        BackendError::new(
            BackendErrorKind::EmissionFailure,
            format!("failed to create backend build root '{}': {error}", build_root.display()),
        )
    })?;
    Ok(build_root)
}

pub fn build_generated_crate(crate_root: &Path) -> BackendResult<PathBuf> {
    let manifest_path = crate_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--release")
        .output()
        .map_err(|error| {
            BackendError::new(
                BackendErrorKind::BuildFailure,
                format!("failed to launch cargo build for '{}': {error}", manifest_path.display()),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(BackendError::new(
            BackendErrorKind::BuildFailure,
            format!(
                "cargo build failed for '{}'\nstdout:\n{}\nstderr:\n{}",
                manifest_path.display(),
                stdout.trim(),
                stderr.trim()
            ),
        ));
    }

    let package_name = crate_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            BackendError::new(
                BackendErrorKind::BuildFailure,
                format!("generated crate root '{}' does not have a valid package name", crate_root.display()),
            )
        })?;
    let binary_path = crate_root.join("target").join("release").join(package_name);
    if !binary_path.exists() {
        return Err(BackendError::new(
            BackendErrorKind::BuildFailure,
            format!("cargo build succeeded but '{}' is missing", binary_path.display()),
        ));
    }

    Ok(binary_path)
}

pub fn emit_backend_artifact(
    session: &BackendSession,
    config: &BackendConfig,
    output_root: &Path,
) -> BackendResult<BackendArtifact> {
    let build_root = prepare_generated_build_dir(output_root)?;
    let source_artifact = emit_generated_crate_skeleton(session)?;
    let crate_root = write_generated_crate(&build_root, &source_artifact)?;

    if matches!(config.mode, BackendMode::EmitSource) {
        let BackendArtifact::RustSourceCrate { files, .. } = source_artifact else {
            unreachable!("generated crate skeleton should stay a Rust source crate");
        };
        return Ok(BackendArtifact::RustSourceCrate {
            root: crate_root.display().to_string(),
            files,
        });
    }

    let built_binary = build_generated_crate(&crate_root)?;
    let final_binary_dir = output_root.join("bin");
    fs::create_dir_all(&final_binary_dir).map_err(|error| {
        BackendError::new(
            BackendErrorKind::BuildFailure,
            format!("failed to create backend binary dir '{}': {error}", final_binary_dir.display()),
        )
    })?;
    let final_binary = final_binary_dir.join(
        built_binary
            .file_name()
            .ok_or_else(|| {
                BackendError::new(
                    BackendErrorKind::BuildFailure,
                    format!("built binary '{}' does not have a file name", built_binary.display()),
                )
            })?,
    );
    fs::copy(&built_binary, &final_binary).map_err(|error| {
        BackendError::new(
            BackendErrorKind::BuildFailure,
            format!(
                "failed to copy built binary '{}' to '{}': {error}",
                built_binary.display(),
                final_binary.display()
            ),
        )
    })?;

    if !config.keep_build_dir {
        fs::remove_dir_all(&crate_root).map_err(|error| {
            BackendError::new(
                BackendErrorKind::BuildFailure,
                format!("failed to remove generated crate dir '{}': {error}", crate_root.display()),
            )
        })?;
    }

    Ok(BackendArtifact::CompiledBinary {
        crate_root: crate_root.display().to_string(),
        binary_path: final_binary.display().to_string(),
    })
}

pub fn summarize_emitted_artifact(artifact: &BackendArtifact) -> String {
    match artifact {
        BackendArtifact::RustSourceCrate { root, files } => format!(
            "generated Rust crate root={root} files={}",
            files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        BackendArtifact::CompiledBinary {
            crate_root,
            binary_path,
        } => format!("compiled backend artifact crate_root={crate_root} binary={binary_path}"),
    }
}

fn runtime_dependency_path() -> PathBuf {
    if let Some(path) = std::env::var_os("FOL_BACKEND_RUNTIME_PATH") {
        return PathBuf::from(path);
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("fol-runtime")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntryCallable {
    rust_path: String,
    recoverable: bool,
}

fn resolve_entry_callable(
    session: &BackendSession,
    entry_candidate: &fol_lower::LoweredEntryCandidate,
) -> Option<EntryCallable> {
    let package = session.workspace().package(&entry_candidate.package_identity)?;
    let routine = package.routine_decls.get(&entry_candidate.routine_id)?;
    if routine.receiver_type.is_some() || !routine.params.is_empty() {
        return None;
    }
    let signature_id = routine.signature?;
    let signature = match session.workspace().type_table().get(signature_id) {
        Some(LoweredType::Routine(signature)) => signature,
        _ => return None,
    };
    let source_unit_id = routine.source_unit_id?;
    let namespace_plan = plan_namespace_layouts(session)
        .into_iter()
        .find(|plan| {
            plan.package_identity == entry_candidate.package_identity
                && plan.source_unit_ids.contains(&source_unit_id)
        })?;
    if render_routine_definition(
        session.workspace(),
        &entry_candidate.package_identity,
        routine,
        session.workspace().type_table(),
    )
    .is_err()
    {
        return None;
    }
    let namespace_path = namespace_plan
        .relative_file
        .trim_end_matches(".rs")
        .replace('/', "::");
    Some(EntryCallable {
        rust_path: format!(
            "packages::{}::{}::{}",
            mangle_package_module_name(&entry_candidate.package_identity),
            namespace_path,
            mangle_routine_name(
                &entry_candidate.package_identity,
                entry_candidate.routine_id,
                &entry_candidate.name
            )
        ),
        recoverable: signature.error_type.is_some(),
    })
}

fn render_namespace_items(
    session: &BackendSession,
    namespace_plan: &crate::NamespaceLayoutPlan,
) -> String {
    let Some(package) = session.workspace().package(&namespace_plan.package_identity) else {
        return String::new();
    };
    let mut items = Vec::new();

    let mut types = package
        .type_decls
        .values()
        .filter(|type_decl| namespace_plan.source_unit_ids.contains(&type_decl.source_unit_id))
        .cloned()
        .collect::<Vec<_>>();
    types.sort_by_key(|type_decl| type_decl.runtime_type.0);
    for type_decl in &types {
        let rendered = match &type_decl.kind {
            fol_lower::LoweredTypeDeclKind::Record { .. } => render_record_definition(
                &namespace_plan.package_identity,
                type_decl,
                session.workspace().type_table(),
            )
            .and_then(|definition| {
                Ok(format!(
                    "{definition}\n{}",
                    render_record_trait_impl(&namespace_plan.package_identity, type_decl)?
                ))
            }),
            fol_lower::LoweredTypeDeclKind::Entry { .. } => render_entry_definition(
                &namespace_plan.package_identity,
                type_decl,
                session.workspace().type_table(),
            )
            .and_then(|definition| {
                Ok(format!(
                    "{definition}\n{}",
                    render_entry_trait_impl(&namespace_plan.package_identity, type_decl)?
                ))
            }),
            fol_lower::LoweredTypeDeclKind::Alias { .. } => Ok(String::new()),
        };
        if let Ok(rendered) = rendered {
            if !rendered.is_empty() {
                items.push(rendered);
            }
        }
    }

    let mut globals = package
        .global_decls
        .values()
        .filter(|global| namespace_plan.source_unit_ids.contains(&global.source_unit_id))
        .cloned()
        .collect::<Vec<_>>();
    globals.sort_by_key(|global| global.id.0);
    for global in &globals {
        if let Ok(rendered) = render_global_declaration(
            &namespace_plan.package_identity,
            global,
            session.workspace().type_table(),
        ) {
            items.push(rendered);
        }
    }

    let mut routines = package
        .routine_decls
        .values()
        .filter(|routine| {
            routine
                .source_unit_id
                .is_some_and(|source_unit_id| namespace_plan.source_unit_ids.contains(&source_unit_id))
        })
        .cloned()
        .collect::<Vec<_>>();
    routines.sort_by_key(|routine| routine.id.0);

    items.extend(routines.iter().filter_map(|routine| {
        render_routine_definition(
            session.workspace(),
            &namespace_plan.package_identity,
            routine,
            session.workspace().type_table(),
        )
        .or_else(|_| {
            render_routine_shell(
                &namespace_plan.package_identity,
                routine,
                session.workspace().type_table(),
            )
        })
        .ok()
    }));

    items.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{
        build_generated_crate, emit_backend_artifact, emit_cargo_toml,
        emit_generated_crate_skeleton, emit_main_rs, emit_namespace_module_shells,
        emit_package_module_shells, prepare_generated_build_dir, summarize_emitted_artifact,
        write_generated_crate,
    };
    use crate::{
        testing::{
            lowered_workspace_from_entry_path, lowered_workspace_from_entry_path_with_config,
            sample_lowered_workspace,
        },
        BackendArtifact, BackendConfig, BackendMode, BackendSession,
    };
    use fol_package::PackageConfig;
    use fol_resolver::ResolverConfig;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("fol_backend_{label}_{unique}"))
    }

    fn write_fixture(root: &Path, source: &str) -> PathBuf {
        fs::create_dir_all(root).expect("backend fixture root");
        let fixture = root.join("main.fol");
        fs::write(&fixture, source).expect("backend fixture source");
        fixture
    }

    fn build_and_run_fixture(source: &str) -> std::process::Output {
        let fixture_root = temp_root("exec");
        let fixture = write_fixture(&fixture_root, source);
        let lowered = lowered_workspace_from_entry_path(&fixture);
        let session = BackendSession::new(lowered);
        let artifact = emit_backend_artifact(
            &session,
            &BackendConfig {
                mode: BackendMode::BuildArtifact,
                keep_build_dir: true,
                ..BackendConfig::default()
            },
            &fixture_root,
        )
        .expect("backend artifact");
        let BackendArtifact::CompiledBinary { binary_path, .. } = artifact else {
            panic!("expected compiled binary artifact");
        };
        let output = Command::new(&binary_path)
            .output()
            .expect("run emitted binary");
        let _ = fs::remove_dir_all(&fixture_root);
        output
    }

    fn build_and_run_workspace(
        entry_path: &Path,
        package_config: PackageConfig,
        resolver_config: ResolverConfig,
    ) -> std::process::Output {
        let lowered = lowered_workspace_from_entry_path_with_config(
            entry_path,
            package_config,
            resolver_config,
        );
        let session = BackendSession::new(lowered);
        let output_root = temp_root("workspace_exec");
        let artifact = emit_backend_artifact(
            &session,
            &BackendConfig {
                mode: BackendMode::BuildArtifact,
                keep_build_dir: true,
                ..BackendConfig::default()
            },
            &output_root,
        )
        .expect("backend artifact");
        let BackendArtifact::CompiledBinary { binary_path, .. } = artifact else {
            panic!("expected compiled binary artifact");
        };
        let output = Command::new(&binary_path)
            .output()
            .expect("run emitted binary");
        let _ = fs::remove_dir_all(&output_root);
        output
    }

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

        let emitted = emit_main_rs(&session).expect("main.rs");

        assert_eq!(emitted.path, "src/main.rs");
        assert_eq!(emitted.module_name, "main");
        assert!(emitted.contents.contains("use fol_runtime::prelude as rt;"));
        assert!(emitted.contents.contains("mod packages;"));
        assert!(emitted.contents.contains("let _entry_package = \"app\";"));
        assert!(emitted.contents.contains("let _entry_name = \"main\";"));
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

    #[test]
    fn namespace_module_shell_emission_keeps_runtime_imports_and_namespace_markers() {
        let session = BackendSession::new(sample_lowered_workspace());

        let emitted = emit_namespace_module_shells(&session);

        assert_eq!(emitted.len(), 4);
        assert_eq!(emitted[0].path, "src/packages/pkg__entry__app/root.rs");
        assert!(emitted[0].contents.contains("use fol_runtime::prelude as rt;"));
        assert!(emitted[0].contents.contains("NAMESPACE_NAME: &str = \"app\""));
        assert!(emitted[0].contents.contains("SOURCE_UNIT_IDS: &[usize] = &[0]"));
        assert_eq!(emitted[1].path, "src/packages/pkg__entry__app/math.rs");
        assert!(emitted[1].contents.contains("NAMESPACE_NAME: &str = \"app::math\""));
        assert_eq!(emitted[3].path, "src/packages/pkg__local__shared/util.rs");
        assert!(emitted[3].contents.contains("NAMESPACE_NAME: &str = \"shared::util\""));
    }

    #[test]
    fn generated_crate_skeleton_snapshot_stays_stable_for_foundation_backend_shape() {
        let session = BackendSession::new(sample_lowered_workspace());

        let artifact = emit_generated_crate_skeleton(&session).expect("artifact");

        let BackendArtifact::RustSourceCrate { root, files } = artifact else {
            panic!("expected RustSourceCrate artifact");
        };

        let snapshot = files
            .iter()
            .map(|file| format!("== {} ==\n{}", file.path, file.contents))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(root.starts_with("fol-build-app-"));
        assert_eq!(files.len(), 8);
        assert!(snapshot.contains("== Cargo.toml =="));
        assert!(snapshot.contains("== src/main.rs =="));
        assert!(snapshot.contains("== src/packages/mod.rs =="));
        assert!(snapshot.contains("== src/packages/pkg__entry__app/mod.rs =="));
        assert!(snapshot.contains("== src/packages/pkg__local__shared/root.rs =="));
        assert!(snapshot.contains("use fol_runtime::prelude as rt;"));
        assert!(snapshot.contains("pub mod pkg__entry__app;"));
        assert!(snapshot.contains("NAMESPACE_NAME: &str = \"shared::util\""));
    }

    #[test]
    fn generated_crate_writer_materializes_files_under_backend_build_root() {
        let session = BackendSession::new(sample_lowered_workspace());
        let artifact = emit_generated_crate_skeleton(&session).expect("artifact");
        let temp_root = temp_root("write");
        let build_root = prepare_generated_build_dir(&temp_root).expect("build root");

        let crate_root = write_generated_crate(&build_root, &artifact).expect("write crate");

        assert!(crate_root.ends_with(session.workspace_identity().crate_dir_name.as_str()));
        assert!(crate_root.join("Cargo.toml").exists());
        assert!(crate_root.join("src/main.rs").exists());
        assert!(crate_root.join("src/packages/mod.rs").exists());

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn prepare_generated_build_dir_creates_the_expected_backend_root() {
        let temp_root = temp_root("build_root");

        let build_root = prepare_generated_build_dir(&temp_root).expect("prepare build root");

        assert!(build_root.ends_with("fol-backend"));
        assert!(build_root.exists());

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn cargo_build_support_compiles_the_generated_crate_skeleton() {
        let session = BackendSession::new(sample_lowered_workspace());
        let artifact = emit_generated_crate_skeleton(&session).expect("artifact");
        let temp_root = temp_root("cargo_build");
        let build_root = prepare_generated_build_dir(&temp_root).expect("build root");
        let crate_root = write_generated_crate(&build_root, &artifact).expect("write crate");

        let binary = build_generated_crate(&crate_root).expect("cargo build");

        assert!(binary.exists());
        assert!(binary.ends_with(session.workspace_identity().crate_dir_name.as_str()));

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn cargo_failure_diagnostics_surface_missing_manifest_context() {
        let temp_root = temp_root("cargo_fail");
        fs::create_dir_all(&temp_root).expect("temp root");

        let error = build_generated_crate(&temp_root).expect_err("missing manifest should fail");

        assert_eq!(error.kind(), crate::BackendErrorKind::BuildFailure);
        assert!(error.message().contains("Cargo.toml"));

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn emit_backend_artifact_honors_emit_source_and_build_artifact_modes() {
        let session = BackendSession::new(sample_lowered_workspace());
        let temp_root = temp_root("modes");

        let emitted = emit_backend_artifact(
            &session,
            &BackendConfig {
                mode: BackendMode::EmitSource,
                ..BackendConfig::default()
            },
            &temp_root,
        )
        .expect("emit source");
        let built = emit_backend_artifact(
            &session,
            &BackendConfig {
                mode: BackendMode::BuildArtifact,
                keep_build_dir: true,
                ..BackendConfig::default()
            },
            &temp_root,
        )
        .expect("build artifact");

        assert!(matches!(emitted, BackendArtifact::RustSourceCrate { .. }));
        assert!(matches!(built, BackendArtifact::CompiledBinary { .. }));

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn emit_backend_artifact_respects_keep_build_dir_and_summary_output() {
        let session = BackendSession::new(sample_lowered_workspace());
        let temp_root = temp_root("keep");
        let artifact = emit_backend_artifact(
            &session,
            &BackendConfig {
                mode: BackendMode::BuildArtifact,
                keep_build_dir: true,
                ..BackendConfig::default()
            },
            &temp_root,
        )
        .expect("build artifact");

        let summary = summarize_emitted_artifact(&artifact);
        let BackendArtifact::CompiledBinary {
            crate_root,
            binary_path,
        } = &artifact else {
            panic!("expected compiled artifact");
        };

        assert!(Path::new(crate_root).exists());
        assert!(Path::new(binary_path).exists());
        assert!(summary.contains("compiled backend artifact"));
        assert!(summary.contains("binary="));

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn full_generated_crate_snapshot_stays_stable_after_backend_materialization() {
        let session = BackendSession::new(sample_lowered_workspace());
        let artifact = emit_generated_crate_skeleton(&session).expect("artifact");

        let summary = summarize_emitted_artifact(&artifact);

        assert!(summary.contains("generated Rust crate root="));
        assert!(summary.contains("Cargo.toml"));
        assert!(summary.contains("src/main.rs"));
        assert!(summary.contains("src/packages/pkg__entry__app/root.rs"));
    }

    #[test]
    fn package_module_shell_emission_adds_nested_mod_files_for_deep_namespaces() {
        let fixture_root = temp_root("deep_namespace_layout");
        let app_root = fixture_root.join("app");
        fs::create_dir_all(app_root.join("api/tools/math")).expect("nested namespace root");
        fs::write(
            app_root.join("main.fol"),
            "fun[] main(): int = {\n    return api::tools::math::leaf()\n}\n",
        )
        .expect("app source");
        fs::write(
            app_root.join("api/tools/math/leaf.fol"),
            "fun[] leaf(): int = {\n    return 7\n}\n",
        )
        .expect("nested source");

        let lowered = lowered_workspace_from_entry_path(&app_root);
        let session = BackendSession::new(lowered);
        let emitted = emit_package_module_shells(&session);

        assert!(emitted
            .iter()
            .any(|file| file.path.ends_with("pkg__entry__app/api/mod.rs")
                && file.contents.contains("pub mod tools;")));
        assert!(emitted
            .iter()
            .any(|file| file.path.ends_with("pkg__entry__app/api/tools/mod.rs")
                && file.contents.contains("pub mod math;")));

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn generated_crate_artifact_file_order_stays_deterministic() {
        let session = BackendSession::new(sample_lowered_workspace());
        let artifact = emit_generated_crate_skeleton(&session).expect("artifact");

        let BackendArtifact::RustSourceCrate { files, .. } = artifact else {
            panic!("expected RustSourceCrate artifact");
        };

        let mut sorted_paths = files.iter().map(|file| file.path.clone()).collect::<Vec<_>>();
        let original_paths = sorted_paths.clone();
        sorted_paths.sort();

        assert_eq!(original_paths, sorted_paths);
    }

    #[test]
    fn executable_backend_runs_scalar_entry_routines_successfully() {
        let output = build_and_run_fixture("fun[] main(): int = {\n    return 7\n}\n");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    }

    #[test]
    fn executable_backend_handles_recoverable_entry_failure_through_process_outcome() {
        let output = build_and_run_fixture(
            "fun[] main(): int / str = {\n    report \"broken\"\n}\n",
        );

        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("broken"));
    }

    #[test]
    fn executable_backend_handles_recoverable_propagation_between_zero_arg_routines() {
        let output = build_and_run_fixture(
            concat!(
                "fun[] load(): int / str = {\n",
                "    report \"bad-input\"\n",
                "}\n",
                "fun[] main(): int / str = {\n",
                "    return load()\n",
                "}\n",
            ),
        );

        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("bad-input"));
    }

    #[test]
    fn executable_backend_runs_container_length_programs() {
        let output = build_and_run_fixture(
            concat!(
                "fun[] main(): int = {\n",
                "    var values: seq[int] = {1, 2, 3}\n",
                "    .echo(.len(values))\n",
                "    return 0\n",
                "}\n",
            ),
        );

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("3"));
    }

    #[test]
    fn executable_backend_runs_echo_programs() {
        let output = build_and_run_fixture(
            concat!(
                "fun[] main(): int = {\n",
                "    .echo(\"hello\")\n",
                "    return 0\n",
                "}\n",
            ),
        );

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello"));
    }

    #[test]
    fn executable_backend_runs_check_programs() {
        let output = build_and_run_fixture(
            concat!(
                "fun[] load(): int / str = {\n",
                "    report \"broken\"\n",
                "}\n",
                "fun[] main(): int = {\n",
                "    .echo(check(load()))\n",
                "    return 0\n",
                "}\n",
            ),
        );

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("true"));
    }

    #[test]
    fn executable_backend_runs_pipe_or_fallback_programs() {
        let output = build_and_run_fixture(
            concat!(
                "fun[] load(): int / str = {\n",
                "    report \"broken\"\n",
                "}\n",
                "fun[] main(): int = {\n",
                "    .echo(load() || 9)\n",
                "    return 0\n",
                "}\n",
            ),
        );

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("9"));
    }

    #[test]
    fn executable_backend_runs_across_loc_std_and_pkg_package_graphs() {
        let fixture_root = temp_root("workspace_graphs");
        let app_root = fixture_root.join("app");
        let shared_root = fixture_root.join("shared");
        let std_root = fixture_root.join("std");
        let pkg_root = fixture_root.join("pkg");
        let pkg_math_root = pkg_root.join("math");

        fs::create_dir_all(&app_root).expect("app root");
        fs::create_dir_all(&shared_root).expect("shared root");
        fs::create_dir_all(std_root.join("fmt")).expect("std root");
        fs::create_dir_all(&pkg_math_root).expect("pkg root");

        fs::write(
            app_root.join("main.fol"),
            concat!(
                "use shared: loc = {\"../shared\"};\n",
                "use fmt: std = {\"fmt\"};\n",
                "use math: pkg = {math};\n",
                "fun[] main(): int = {\n",
                "    .echo(loc_answer)\n",
                "    .echo(std_answer)\n",
                "    .echo(pkg_answer)\n",
                "    return loc_answer + std_answer + pkg_answer\n",
                "}\n",
            ),
        )
        .expect("app source");
        fs::write(shared_root.join("lib.fol"), "var[exp] loc_answer: int = 2\n").expect("shared");
        fs::write(std_root.join("fmt").join("lib.fol"), "var[exp] std_answer: int = 3\n")
            .expect("std");
        fs::write(
            pkg_math_root.join("package.yaml"),
            "name: math\nversion: 0.1.0\n",
        )
        .expect("pkg manifest");
        fs::write(
            pkg_math_root.join("build.fol"),
            "def root: loc = {\"src\"}\n",
        )
        .expect("pkg build");
        fs::create_dir_all(pkg_math_root.join("src")).expect("pkg src");
        fs::write(
            pkg_math_root.join("src").join("lib.fol"),
            "var[exp] pkg_answer: int = 4\n",
        )
        .expect("pkg source");

        let output = build_and_run_workspace(
            &app_root,
            PackageConfig {
                std_root: Some(std_root.display().to_string()),
                package_store_root: Some(pkg_root.display().to_string()),
                package_cache_root: None,
            },
            ResolverConfig {
                std_root: Some(std_root.display().to_string()),
                package_store_root: Some(pkg_root.display().to_string()),
            },
        );

        let _ = fs::remove_dir_all(&fixture_root);

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2"));
        assert!(stdout.contains("3"));
        assert!(stdout.contains("4"));
    }
}

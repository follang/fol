#[cfg(test)]
mod tests {
    use crate::emit::{
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
        assert!(emitted.contents.contains(&format!(
            "name = \"{}\"",
            session.workspace_identity().crate_dir_name
        )));
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
        assert!(emitted[0]
            .contents
            .contains("use fol_runtime::prelude as rt;"));
        assert!(emitted[0]
            .contents
            .contains("NAMESPACE_NAME: &str = \"app\""));
        assert!(emitted[0]
            .contents
            .contains("SOURCE_UNIT_IDS: &[usize] = &[0]"));
        assert_eq!(emitted[1].path, "src/packages/pkg__entry__app/math.rs");
        assert!(emitted[1]
            .contents
            .contains("NAMESPACE_NAME: &str = \"app::math\""));
        assert_eq!(emitted[3].path, "src/packages/pkg__local__shared/util.rs");
        assert!(emitted[3]
            .contents
            .contains("NAMESPACE_NAME: &str = \"shared::util\""));
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
        } = &artifact
        else {
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
        assert!(emitted.iter().any(
            |file| file.path.ends_with("pkg__entry__app/api/tools/mod.rs")
                && file.contents.contains("pub mod math;")
        ));

        let _ = fs::remove_dir_all(&fixture_root);
    }

    #[test]
    fn generated_crate_artifact_file_order_stays_deterministic() {
        let session = BackendSession::new(sample_lowered_workspace());
        let artifact = emit_generated_crate_skeleton(&session).expect("artifact");

        let BackendArtifact::RustSourceCrate { files, .. } = artifact else {
            panic!("expected RustSourceCrate artifact");
        };

        let mut sorted_paths = files
            .iter()
            .map(|file| file.path.clone())
            .collect::<Vec<_>>();
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
        let output =
            build_and_run_fixture("fun[] main(): int / str = {\n    report \"broken\"\n}\n");

        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("broken"));
    }

    #[test]
    fn executable_backend_handles_recoverable_propagation_between_zero_arg_routines() {
        let output = build_and_run_fixture(concat!(
            "fun[] load(): int / str = {\n",
            "    report \"bad-input\"\n",
            "}\n",
            "fun[] main(): int / str = {\n",
            "    return load()\n",
            "}\n",
        ));

        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("bad-input"));
    }

    #[test]
    fn executable_backend_runs_container_length_programs() {
        let output = build_and_run_fixture(concat!(
            "fun[] main(): int = {\n",
            "    var values: seq[int] = {1, 2, 3}\n",
            "    .echo(.len(values))\n",
            "    return 0\n",
            "}\n",
        ));

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("3"));
    }

    #[test]
    fn executable_backend_runs_echo_programs() {
        let output = build_and_run_fixture(concat!(
            "fun[] main(): int = {\n",
            "    .echo(\"hello\")\n",
            "    return 0\n",
            "}\n",
        ));

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello"));
    }

    #[test]
    fn executable_backend_runs_check_programs() {
        let output = build_and_run_fixture(concat!(
            "fun[] load(): int / str = {\n",
            "    report \"broken\"\n",
            "}\n",
            "fun[] main(): int = {\n",
            "    .echo(check(load()))\n",
            "    return 0\n",
            "}\n",
        ));

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("true"));
    }

    #[test]
    fn executable_backend_runs_pipe_or_fallback_programs() {
        let output = build_and_run_fixture(concat!(
            "fun[] load(): int / str = {\n",
            "    report \"broken\"\n",
            "}\n",
            "fun[] main(): int = {\n",
            "    .echo(load() || 9)\n",
            "    return 0\n",
            "}\n",
        ));

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
        fs::write(
            shared_root.join("lib.fol"),
            "var[exp] loc_answer: int = 2\n",
        )
        .expect("shared");
        fs::write(
            std_root.join("fmt").join("lib.fol"),
            "var[exp] std_answer: int = 3\n",
        )
        .expect("std");
        fs::write(
            pkg_math_root.join("package.yaml"),
            "name: math\nversion: 0.1.0\n",
        )
        .expect("pkg manifest");
        fs::write(
            pkg_math_root.join("build.fol"),
            "pro[] build(graph: Graph): non = {\n    return graph\n}\n",
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

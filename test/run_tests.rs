// Main test runner for FOL compiler components

mod stream {
    include!("stream/test_stream.rs");
}

mod lexer {
    include!("lexer/test_lexer.rs");
}

mod parser {
    include!("parser/test_parser.rs");
}

mod resolver {
    include!("resolver/test_resolver.rs");
}

mod typecheck {
    include!("typecheck/test_typecheck.rs");
}

mod apps {
    include!("apps/test_apps.rs");
}

#[cfg(test)]
mod integration_tests {
    use fol_editor::{
        EditorConfig, EditorLspServer, JsonRpcId, JsonRpcNotification, JsonRpcRequest,
        LspCompletionList, LspCompletionParams, LspDidOpenTextDocumentParams, LspDocumentSymbol,
        LspDocumentSymbolParams, LspPosition, LspTextDocumentIdentifier, LspTextDocumentItem,
    };
    use fol_package::{
        evaluate_build_source, infer_package_root, parse_directory_package_syntax,
        parse_package_build, BuildEvaluationInputs, BuildEvaluationRequest, BuildOptimizeMode,
        PackageBuildMode, PackageSourceKind,
    };
    use serde_json::Value;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fol_integration_{}_{}_{}",
            label,
            std::process::id(),
            stamp
        ))
    }

    fn run_fol(args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_fol"))
            .args(args)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Should run fol CLI")
    }

    fn run_fol_in_dir(dir: &Path, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_fol"))
            .args(args)
            .current_dir(dir)
            .output()
            .expect("Should run fol CLI in directory")
    }

    fn run_fol_with_env(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fol"));
        command.args(args);
        command.current_dir(env!("CARGO_MANIFEST_DIR"));
        for (key, value) in envs {
            command.env(key, value);
        }
        command.output().expect("Should run fol CLI with env")
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn example_package_roots() -> Vec<PathBuf> {
        let root = repo_root().join("test/app/formal");
        vec![
            root.join("exe_basic"),
            root.join("static_lib"),
            root.join("shared_lib"),
            root.join("generated_file"),
            root.join("dependency_workspace/app"),
            root.join("dependency_workspace/shared"),
        ]
    }

    fn build_fixture_root(name: &str) -> PathBuf {
        repo_root().join("test/app/build").join(name)
    }

    fn parse_cli_json(output: &std::process::Output) -> Value {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_start = stdout
            .find('{')
            .expect("CLI JSON output should contain a JSON object");
        serde_json::from_str(&stdout[json_start..]).expect("CLI JSON output should stay valid")
    }

    fn open_lsp_document(server: &mut EditorLspServer, uri: String, text: &str) {
        let diagnostics = server
            .handle_notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "textDocument/didOpen".to_string(),
                params: Some(
                    serde_json::to_value(LspDidOpenTextDocumentParams {
                        text_document: LspTextDocumentItem {
                            uri,
                            language_id: "fol".to_string(),
                            version: 1,
                            text: text.to_string(),
                        },
                    })
                    .expect("LSP didOpen params should serialize"),
                ),
            })
            .expect("LSP didOpen should succeed");

        assert_eq!(diagnostics.len(), 1);
    }

    fn write_combined_lowering_repro_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "var enabled: bol = true;\n",
                "var default_name: str = \"Ada\";\n",
                "var low_count: int = 1;\n",
                "var high_count: int = 7;\n",
                "typ NameTag: rec = {\n",
                "    label: str;\n",
                "    code: int\n",
                "};\n",
                "typ Audit: rec = {\n",
                "    active: bol;\n",
                "    marker: NameTag\n",
                "};\n",
                "typ User: rec = {\n",
                "    name: str;\n",
                "    count: int;\n",
                "    audit: Audit\n",
                "};\n",
                "fun[] build_tag(): NameTag = {\n",
                "    return { label = \"stable\", code = high_count };\n",
                "};\n",
                "fun[] build_user(flag: bol): User = {\n",
                "    return {\n",
                "        name = default_name,\n",
                "        count = high_count,\n",
                "        audit = {\n",
                "            active = flag,\n",
                "            marker = build_tag(),\n",
                "        },\n",
                "    };\n",
                "};\n",
                "fun[] choose_count(flag: bol): int = {\n",
                "    when(flag) {\n",
                "        case(true) { return high_count }\n",
                "        * { return low_count }\n",
                "    }\n",
                "};\n",
                "fun[] main(flag: bol): int = {\n",
                "    var current: User = build_user(flag);\n",
                "    var names: seq[str] = {\"Ada\", \"Lin\"};\n",
                "    var counts: map[str, int] = {{\"ada\", 1}, {\"lin\", 2}};\n",
                "    loop(flag) {\n",
                "        break;\n",
                "    }\n",
                "    when(flag) {\n",
                "        case(true) { return current.audit.marker.code; }\n",
                "        * { return counts[\"lin\"]; }\n",
                "    }\n",
                "};\n",
            ),
        )
        .expect("Should write combined lowering repro fixture");
        fixture
    }

    fn write_parameter_scope_lowering_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "fun[] choose(flag: bol): int = {\n",
                "    when(flag) {\n",
                "        case(true) { return 1 }\n",
                "        * { return 0 }\n",
                "    }\n",
                "};\n",
                "fun[] echo(flag: bol): bol = {\n",
                "    return flag;\n",
                "};\n",
                "fun[] main(flag: bol): int = {\n",
                "    when(echo(flag)) {\n",
                "        case(true) { return choose(flag) }\n",
                "        * { return 0 }\n",
                "    }\n",
                "};\n",
            ),
        )
        .expect("Should write parameter-scope lowering fixture");
        fixture
    }

    fn write_container_lowering_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "fun[] main(): int = {\n",
                "    var names: seq[str] = {\"Ada\", \"Lin\"};\n",
                "    var counts: map[str, int] = {{\"ada\", 1}, {\"lin\", 2}};\n",
                "    return counts[\"lin\"];\n",
                "};\n",
            ),
        )
        .expect("Should write container lowering fixture");
        fixture
    }

    fn write_early_return_when_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "fun[] main(flag: bol): int = {\n",
                "    when(flag) {\n",
                "        case(true) { return 7 }\n",
                "        * { return 3 }\n",
                "    }\n",
                "};\n",
            ),
        )
        .expect("Should write early-return when lowering fixture");
        fixture
    }

    fn write_backend_scalar_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!("fun[] main(): int = {\n", "    return 7;\n", "};\n",),
        )
        .expect("Should write backend scalar fixture");
        fixture
    }

    fn semantic_bin_build(name: &str) -> String {
        format!(
            concat!(
                "pro[] build(graph: Graph): non = {{\n",
                "    var app = graph.add_exe({{ name = \"{name}\", root = \"src/main.fol\" }});\n",
                "    graph.install(app);\n",
                "    graph.add_run(app);\n",
                "}};\n",
            ),
            name = name
        )
    }

    fn semantic_lib_build(name: &str) -> String {
        format!(
            concat!(
                "pro[] build(graph: Graph): non = {{\n",
                "    var lib = graph.add_static_lib({{ name = \"{name}\", root = \"src/lib.fol\" }});\n",
                "    graph.install(lib);\n",
                "}};\n",
            ),
            name = name
        )
    }

    fn create_git_package_repo(root: &Path, name: &str, version: &str) {
        std::fs::create_dir_all(root.join("src")).expect("Should create git package source dir");
        std::fs::write(
            root.join("package.yaml"),
            format!("name: {name}\nversion: {version}\n"),
        )
        .expect("Should write git package metadata");
        std::fs::write(root.join("build.fol"), semantic_lib_build(name))
            .expect("Should write git package build");
        std::fs::write(root.join("src/lib.fol"), "var[exp] level: int = 1;\n")
            .expect("Should write git package source");

        for args in [
            vec!["init"],
            vec!["config", "user.name", "FOL"],
            vec!["config", "user.email", "fol@example.com"],
            vec!["add", "."],
            vec!["commit", "-m", "init"],
        ] {
            let status = Command::new("git")
                .args(&args)
                .current_dir(root)
                .status()
                .expect("Should run git command for fixture repo");
            assert!(status.success(), "git {:?} should succeed", args);
        }
    }

    fn create_app_with_git_dependency(app_root: &Path, remote_root: &Path) {
        std::fs::create_dir_all(app_root.join("src")).expect("Should create app source dir");
        std::fs::write(
            app_root.join("package.yaml"),
            format!(
                "name: {}\nversion: 0.1.0\ndep.logtiny: git:git+file://{}\n",
                app_root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("app"),
                remote_root.display()
            ),
        )
        .expect("Should write app manifest");
        std::fs::write(app_root.join("build.fol"), semantic_bin_build("app"))
            .expect("Should write app build");
        std::fs::write(
            app_root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0;\n};\n",
        )
        .expect("Should write app source");
    }

    fn read_lock_revision(lockfile: &Path) -> String {
        let parsed = fol_package::parse_package_lockfile(
            &std::fs::read_to_string(lockfile).expect("Should read lockfile"),
        )
        .expect("Lockfile should parse");
        parsed.entries[0].selected_revision.clone()
    }


    #[cfg(test)]
    #[path = "integration_pipeline.rs"]
    mod pipeline;

    #[cfg(test)]
    #[path = "integration_cli_compile.rs"]
    mod cli_compile;

    #[cfg(test)]
    #[path = "integration_cli_lowering.rs"]
    mod cli_lowering;

    #[cfg(test)]
    #[path = "integration_cli_errors.rs"]
    mod cli_errors;

    #[cfg(test)]
    #[path = "integration_cli_typecheck.rs"]
    mod cli_typecheck;

    #[cfg(test)]
    #[path = "integration_editor_and_build.rs"]
    mod editor_and_build;

    #[cfg(test)]
    #[path = "integration_editor_sync.rs"]
    mod editor_sync;

    #[cfg(test)]
    #[path = "integration_diagnostics_pipeline.rs"]
    mod diagnostics_pipeline;

    #[cfg(test)]
    mod language_facts {
        #[test]
        fn builtin_type_names_are_nonempty_and_unique() {
            let names = fol_typecheck::BuiltinType::ALL_NAMES;
            assert!(names.len() >= 6);
            let mut seen = std::collections::HashSet::new();
            for name in names {
                assert!(!name.is_empty(), "builtin type name must not be empty");
                assert!(seen.insert(name), "duplicate builtin type name: {name}");
            }
        }

        #[test]
        fn declaration_keywords_are_nonempty_and_unique() {
            let keywords = fol_lexer::token::buildin::DECLARATION_KEYWORDS;
            assert!(keywords.len() >= 12);
            let mut seen = std::collections::HashSet::new();
            for kw in keywords {
                assert!(!kw.is_empty());
                assert!(seen.insert(kw), "duplicate declaration keyword: {kw}");
            }
        }

        #[test]
        fn source_kind_names_are_canonical() {
            let kinds = fol_parser::SOURCE_KIND_NAMES;
            assert_eq!(kinds.len(), 3);
            assert!(kinds.contains(&"loc"));
            assert!(kinds.contains(&"std"));
            assert!(kinds.contains(&"pkg"));
        }

        #[test]
        fn container_and_shell_type_names_are_canonical() {
            let containers = fol_parser::CONTAINER_TYPE_NAMES;
            let shells = fol_parser::SHELL_TYPE_NAMES;
            assert!(containers.contains(&"vec"));
            assert!(containers.contains(&"map"));
            assert!(shells.contains(&"opt"));
            assert!(shells.contains(&"err"));
        }

        #[test]
        fn intrinsic_registry_has_entries() {
            let registry = fol_intrinsics::intrinsic_registry();
            assert!(!registry.is_empty(), "intrinsic registry must not be empty");
            for entry in registry {
                assert!(!entry.name.is_empty(), "intrinsic name must not be empty");
            }
        }
    }

    #[cfg(test)]
    mod treesitter_sync {
        use super::*;
        use std::collections::BTreeSet;

        fn highlights_scm() -> String {
            let path = repo_root()
                .join("lang/tooling/fol-editor/queries/fol/highlights.scm");
            std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("should read highlights.scm: {e}"))
        }

        fn extract_match_regex_names(text: &str, pattern: &str) -> BTreeSet<String> {
            let mut names = BTreeSet::new();
            for line in text.lines() {
                if line.contains(pattern) {
                    if let Some(start) = line.find("^(") {
                        if let Some(end) = line[start..].find(")$") {
                            let inner = &line[start + 2..start + end];
                            for name in inner.split('|') {
                                names.insert(name.to_string());
                            }
                        }
                    }
                }
            }
            names
        }

        fn extract_node_label_names(text: &str, node_type: &str, capture: &str) -> BTreeSet<String> {
            let mut names = BTreeSet::new();
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.contains(node_type) && trimmed.contains(capture) {
                    if let Some(start) = trimmed.find('"') {
                        let rest = &trimmed[start + 1..];
                        if let Some(end) = rest.find('"') {
                            names.insert(rest[..end].to_string());
                        }
                    }
                }
            }
            names
        }

        #[test]
        fn highlights_builtin_types_match_compiler_constants() {
            let scm = highlights_scm();
            let scm_types = extract_match_regex_names(&scm, "@type.builtin");
            let compiler_types: BTreeSet<String> = fol_typecheck::BuiltinType::ALL_NAMES
                .iter()
                .map(|s| s.to_string())
                .collect();
            assert_eq!(
                scm_types, compiler_types,
                "highlights.scm builtin type regex must match BuiltinType::ALL_NAMES"
            );
        }

        #[test]
        fn highlights_dot_intrinsics_match_implemented_registry() {
            let scm = highlights_scm();
            let scm_intrinsics = extract_match_regex_names(&scm, "@function.builtin");
            let compiler_intrinsics: BTreeSet<String> = fol_intrinsics::intrinsic_registry()
                .iter()
                .filter(|e| {
                    e.surface == fol_intrinsics::IntrinsicSurface::DotRootCall
                        && e.status == fol_intrinsics::IntrinsicStatus::Implemented
                })
                .map(|e| e.name.to_string())
                .collect();
            assert_eq!(
                scm_intrinsics, compiler_intrinsics,
                "highlights.scm dot-intrinsic regex must match implemented DotRootCall intrinsics"
            );
        }

        #[test]
        fn highlights_container_types_match_compiler_constants() {
            let scm = highlights_scm();
            let scm_containers = extract_node_label_names(&scm, "container_type", "@type.builtin");
            let compiler_containers: BTreeSet<String> = fol_parser::CONTAINER_TYPE_NAMES
                .iter()
                .map(|s| s.to_string())
                .collect();
            assert_eq!(
                scm_containers, compiler_containers,
                "highlights.scm container types must match CONTAINER_TYPE_NAMES"
            );
        }

        #[test]
        fn highlights_shell_types_match_compiler_constants() {
            let scm = highlights_scm();
            let scm_shells = extract_node_label_names(&scm, "shell_type", "@type.builtin");
            let compiler_shells: BTreeSet<String> = fol_parser::SHELL_TYPE_NAMES
                .iter()
                .map(|s| s.to_string())
                .collect();
            assert_eq!(
                scm_shells, compiler_shells,
                "highlights.scm shell types must match SHELL_TYPE_NAMES"
            );
        }

        #[test]
        fn highlights_source_kinds_match_compiler_constants() {
            let scm = highlights_scm();
            let scm_kinds = extract_node_label_names(&scm, "source_kind", "@keyword.import");
            let compiler_kinds: BTreeSet<String> = fol_parser::SOURCE_KIND_NAMES
                .iter()
                .map(|s| s.to_string())
                .collect();
            assert_eq!(
                scm_kinds, compiler_kinds,
                "highlights.scm source kinds must match SOURCE_KIND_NAMES"
            );
        }
    }
}

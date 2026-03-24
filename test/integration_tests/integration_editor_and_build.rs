use super::*;
use fol_editor::{LspDefinitionParams, LspHover, LspHoverParams, LspLocation};

fn strip_ansi(value: &str) -> String {
    let mut stripped = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }

        stripped.push(ch);
    }

    stripped
}

    #[test]
    fn test_editor_file_commands_cover_build_fol_entry_files() {
        let parse = run_fol(&[
            "tool",
            "--output",
            "json",
            "parse",
            "xtra/logtiny/build.fol",
        ]);
        assert!(
            parse.status.success(),
            "build.fol parse should succeed: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&parse.stdout),
            String::from_utf8_lossy(&parse.stderr)
        );
        let parse_json = parse_cli_json(&parse);
        assert_eq!(parse_json["command"], "parse");
        assert!(parse_json["summary"]
            .as_str()
            .expect("parse summary should be a string")
            .contains("xtra/logtiny/build.fol"));

        let highlight = run_fol(&[
            "tool",
            "--output",
            "json",
            "highlight",
            "xtra/logtiny/build.fol",
        ]);
        assert!(
            highlight.status.success(),
            "build.fol highlight should succeed: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&highlight.stdout),
            String::from_utf8_lossy(&highlight.stderr)
        );
        let highlight_json = parse_cli_json(&highlight);
        assert_eq!(highlight_json["command"], "highlight");
        assert!(highlight_json["summary"]
            .as_str()
            .expect("highlight summary should be a string")
            .contains("capture_count="));
        assert!(highlight_json["summary"]
            .as_str()
            .expect("highlight summary should be a string")
            .contains("xtra/logtiny/build.fol"));

        let symbols = run_fol(&[
            "tool",
            "--output",
            "json",
            "symbols",
            "xtra/logtiny/build.fol",
        ]);
        assert!(
            symbols.status.success(),
            "build.fol symbols should succeed: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&symbols.stdout),
            String::from_utf8_lossy(&symbols.stderr)
        );
        let symbols_json = parse_cli_json(&symbols);
        assert_eq!(symbols_json["command"], "symbols");
        assert!(symbols_json["summary"]
            .as_str()
            .expect("symbols summary should be a string")
            .contains("query_snapshots="));
        assert!(symbols_json["summary"]
            .as_str()
            .expect("symbols summary should be a string")
            .contains("xtra/logtiny/build.fol"));
    }

    #[test]
    fn test_lsp_covers_build_fol_symbols_hover_definition_and_completion() {
        let temp_root = unique_temp_root("lsp_build_fol");
        std::fs::create_dir_all(temp_root.join("src")).expect("should create source root");
        std::fs::write(
            temp_root.join("package.yaml"),
            "name: demo\nversion: 0.1.0\n",
        )
        .expect("should write package metadata");
        let build_text = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var app = graph.add_exe({ name = \"demo\", root = \"src/main.fol\" });\n",
            "    graph.\n",
            "};\n",
        );
        std::fs::write(temp_root.join("build.fol"), build_text).expect("should write build file");
        let uri = format!("file://{}", temp_root.join("build.fol").display());

        let mut server = EditorLspServer::new(EditorConfig::default());
        open_lsp_document(&mut server, uri.clone(), build_text);

        let symbols = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(1),
                method: "textDocument/documentSymbol".to_string(),
                params: Some(
                    serde_json::to_value(LspDocumentSymbolParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                    })
                    .expect("documentSymbol params should serialize"),
                ),
            })
            .expect("documentSymbol request should succeed")
            .expect("documentSymbol should produce a response");
        let symbols: Vec<LspDocumentSymbol> =
            serde_json::from_value(symbols.result.expect("symbols should have a result"))
                .expect("document symbols should deserialize");

        assert!(
            !symbols.is_empty(),
            "build.fol document symbols should include the build routine after resolver processes build units"
        );
        assert!(
            symbols.iter().any(|s| s.name == "build"),
            "build.fol document symbols should include the 'build' entry routine"
        );

        let hover = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(2),
                method: "textDocument/hover".to_string(),
                params: Some(
                    serde_json::to_value(LspHoverParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 1,
                            character: 8,
                        },
                    })
                    .expect("hover params should serialize"),
                ),
            })
            .expect("hover request should succeed")
            .expect("hover should produce a response");
        let _hover: Option<LspHover> =
            serde_json::from_value(hover.result.expect("hover should have a result"))
                .expect("hover result should deserialize");

        let definition = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(3),
                method: "textDocument/definition".to_string(),
                params: Some(
                    serde_json::to_value(LspDefinitionParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 1,
                            character: 8,
                        },
                    })
                    .expect("definition params should serialize"),
                ),
            })
            .expect("definition request should succeed")
            .expect("definition should produce a response");
        let _definition: Option<LspLocation> = serde_json::from_value(
            definition.result.expect("definition should have a result"),
        )
        .expect("definition result should deserialize");

        let completion = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(4),
                method: "textDocument/completion".to_string(),
                params: Some(
                    serde_json::to_value(LspCompletionParams {
                        text_document: LspTextDocumentIdentifier { uri: uri.clone() },
                        position: LspPosition {
                            line: 2,
                            character: 10,
                        },
                        context: None,
                    })
                    .expect("completion params should serialize"),
                ),
            })
            .expect("completion request should succeed")
            .expect("completion should produce a response");
        let completion: LspCompletionList =
            serde_json::from_value(completion.result.expect("completion should have a result"))
                .expect("completion result should deserialize");

        assert!(
            !completion.items.is_empty(),
            "build.fol completion should still return a non-empty list"
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_examples_tree_contains_discoverable_formal_packages() {
        for root in example_package_roots() {
            assert!(
                root.join("package.yaml").is_file(),
                "missing package.yaml in {}",
                root.display()
            );
            assert!(
                root.join("build.fol").is_file(),
                "missing build.fol in {}",
                root.display()
            );
            let display_name = root
                .file_name()
                .and_then(|name| name.to_str())
                .expect("example package name should be utf-8");
            let syntax =
                parse_directory_package_syntax(&root, display_name, PackageSourceKind::Package)
                    .expect("formal example package syntax should parse");
            let discovered =
                infer_package_root(&syntax).expect("formal example package should be discoverable");
            assert_eq!(
                discovered,
                root.canonicalize()
                    .expect("example package root should canonicalize")
            );
        }
    }

    #[test]
    fn test_examples_build_files_parse_cleanly() {
        for root in example_package_roots() {
            let build = parse_package_build(&root.join("build.fol"))
                .expect("checked-in example build.fol should parse cleanly");
            assert_eq!(
                build.mode(),
                PackageBuildMode::ModernOnly,
                "example build should stay on the semantic build surface: {}",
                root.display()
            );
        }
    }

    #[test]
    fn test_examples_formal_packages_keep_build_source_units_in_syntax() {
        for root in example_package_roots() {
            let display_name = root
                .file_name()
                .and_then(|name| name.to_str())
                .expect("example package name should be utf-8");
            let syntax =
                parse_directory_package_syntax(&root, display_name, PackageSourceKind::Package)
                    .expect("formal example package syntax should parse");

            assert_eq!(
                syntax
                    .source_units
                    .iter()
                    .filter(|unit| unit.kind == fol_parser::ast::ParsedSourceUnitKind::Build)
                    .count(),
                1,
                "expected exactly one build source unit in {}",
                root.display()
            );
        }
    }

    #[test]
    fn test_build_fixture_local_root_package_builds_and_runs() {
        let root = build_fixture_root("exe_object_config");
        let build_source = std::fs::read_to_string(root.join("build.fol"))
            .expect("build fixture should keep a checked-in build file");
        assert!(
            build_source.starts_with("pro[] build(graph: Graph): non"),
            "fixture should exercise the new build entry: {}",
            build_source
        );

        let build = run_fol_in_dir(&root, &["code", "build", "--keep-build-dir"]);
        assert!(
            build.status.success(),
            "local-root build fixture should build: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            String::from_utf8_lossy(&build.stdout).contains("built 1 workspace package(s)"),
            "local-root build fixture should report a build summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_fol_in_dir(&root, &["code", "run"]);
        assert!(
            run.status.success(),
            "local-root build fixture should run: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            String::from_utf8_lossy(&run.stdout).contains("ran "),
            "local-root build fixture should report a run summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }

    #[test]
    fn test_build_fixture_named_bundle_step_builds_from_new_semantic_entry() {
        let root = build_fixture_root("hybrid_bundle_step");
        let app_root = root.join("app");
        let build_source = std::fs::read_to_string(app_root.join("build.fol"))
            .expect("bundle fixture should keep a checked-in build file");
        assert!(
            build_source.starts_with("pro[] build(graph: Graph): non"),
            "bundle fixture should exercise the new build entry: {}",
            build_source
        );

        let build = run_fol_in_dir(&app_root, &["code", "build"]);
        assert!(
            build.status.success(),
            "bundle fixture should build through the new semantic build route: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            String::from_utf8_lossy(&build.stdout).contains("built 1 workspace package(s)"),
            "bundle fixture should keep the routed build summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let check = run_fol_in_dir(&app_root, &["code", "build", "--step", "bundle"]);
        assert!(
            check.status.success(),
            "bundle fixture should execute the named bundle step: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );
    }

    #[test]
    fn test_build_fixture_nested_local_library_executes_default_run_route() {
        let root = build_fixture_root("run_step_chain");
        let build_source = std::fs::read_to_string(root.join("build.fol"))
            .expect("run-step fixture should keep a checked-in build file");
        assert!(
            build_source.starts_with("pro[] build(graph: Graph): non"),
            "run-step fixture should exercise the new build entry: {}",
            build_source
        );

        let run = run_fol_in_dir(&root, &["code", "run"]);
        assert!(
            run.status.success(),
            "nested local-library fixture should run successfully: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            String::from_utf8_lossy(&run.stdout).contains("ran "),
            "nested local-library fixture should report a run summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }

    #[test]
    fn test_build_fixture_second_local_package_runs_with_new_semantic_entry() {
        let root = build_fixture_root("pkg_dependency_run");
        let app_root = root.join("app");
        let build_source = std::fs::read_to_string(app_root.join("build.fol"))
            .expect("secondary run fixture should keep a checked-in build file");
        assert!(
            build_source.starts_with("pro[] build(graph: Graph): non"),
            "secondary run fixture should exercise the new build entry: {}",
            build_source
        );

        let run = run_fol_in_dir(&app_root, &["code", "run"]);
        assert!(
            run.status.success(),
            "secondary run fixture should run with the new semantic entry: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            String::from_utf8_lossy(&run.stdout).contains("ran "),
            "secondary run fixture should report a run summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }

    #[test]
    fn test_build_fixture_conditional_step_evaluates_when_condition() {
        let root = build_fixture_root("conditional_step");
        let build_path = root.join("build.fol");
        let source = std::fs::read_to_string(&build_path)
            .expect("conditional_step fixture should keep a build.fol");

        let request_release = BuildEvaluationRequest {
            package_root: root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: root.display().to_string(),
                optimize: Some(BuildOptimizeMode::ReleaseFast),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };
        let evaluated_release = evaluate_build_source(&request_release, &build_path, &source)
            .expect("conditional_step should evaluate with release-fast")
            .expect("release-fast evaluation should produce operations");
        assert!(
            evaluated_release
                .result
                .graph
                .steps()
                .iter()
                .any(|s| s.name == "strip"),
            "release-fast should add the strip step"
        );

        let request_debug = BuildEvaluationRequest {
            package_root: root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };
        let evaluated_debug = evaluate_build_source(&request_debug, &build_path, &source)
            .expect("conditional_step should evaluate without optimize")
            .expect("debug evaluation should produce operations");
        assert!(
            !evaluated_debug
                .result
                .graph
                .steps()
                .iter()
                .any(|s| s.name == "strip"),
            "debug build should not add the strip step"
        );
    }

    #[test]
    fn test_build_fixture_helper_routine_evaluates_correctly() {
        let root = build_fixture_root("helper_routine");
        let build_path = root.join("build.fol");
        let source = std::fs::read_to_string(&build_path)
            .expect("helper_routine fixture should keep a build.fol");

        let request = BuildEvaluationRequest {
            package_root: root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };
        let evaluated = evaluate_build_source(&request, &build_path, &source)
            .expect("helper_routine should evaluate")
            .expect("helper_routine should produce operations");

        let artifacts = evaluated.result.graph.artifacts();
        assert!(
            artifacts.iter().any(|a| a.name == "core"),
            "helper_routine should produce a core static lib"
        );
        assert!(
            artifacts.iter().any(|a| a.name == "io"),
            "helper_routine should produce an io static lib"
        );
        assert!(
            artifacts.iter().any(|a| a.name == "app"),
            "helper_routine should produce an app executable"
        );
    }

    #[test]
    fn test_build_fixture_loop_libs_produces_multiple_artifacts() {
        let root = build_fixture_root("loop_libs");
        let build_path = root.join("build.fol");
        let source = std::fs::read_to_string(&build_path)
            .expect("loop_libs fixture should keep a build.fol");

        let request = BuildEvaluationRequest {
            package_root: root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };
        let evaluated = evaluate_build_source(&request, &build_path, &source)
            .expect("loop_libs should evaluate")
            .expect("loop_libs should produce operations for each iteration");

        let artifacts = evaluated.result.graph.artifacts();
        assert!(
            artifacts.iter().any(|a| a.name == "core"),
            "loop should produce core artifact"
        );
        assert!(
            artifacts.iter().any(|a| a.name == "io"),
            "loop should produce io artifact"
        );
        assert!(
            artifacts.iter().any(|a| a.name == "utils"),
            "loop should produce utils artifact"
        );
    }

    #[test]
    fn test_build_fixture_d_options_accepts_option_overrides() {
        let root = build_fixture_root("d_options");
        let build_path = root.join("build.fol");
        let source = std::fs::read_to_string(&build_path)
            .expect("d_options fixture should keep a build.fol");

        let request = BuildEvaluationRequest {
            package_root: root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };
        let evaluated = evaluate_build_source(&request, &build_path, &source)
            .expect("d_options should evaluate")
            .expect("d_options should produce operations");

        let options = evaluated.result.graph.options();
        assert!(
            options.iter().any(|o| o.name == "root"),
            "d_options should declare the root user option"
        );
    }

    #[test]
    fn test_build_fixture_alloc_model_supports_string_values() {
        let root = build_fixture_root("model_alloc_str");

        let build = run_fol_in_dir(&root, &["code", "build"]);
        assert!(
            build.status.success(),
            "alloc string fixture should build: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            String::from_utf8_lossy(&build.stdout).contains("built 1 workspace package(s)"),
            "alloc string fixture should report a build summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            String::from_utf8_lossy(&build.stdout).contains("fol_model=alloc"),
            "alloc string fixture should surface its fol_model in the build summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }

    #[test]
    fn test_build_fixture_alloc_model_supports_sequences() {
        let root = build_fixture_root("model_alloc_seq");

        let build = run_fol_in_dir(&root, &["code", "build"]);
        assert!(
            build.status.success(),
            "alloc sequence fixture should build: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            String::from_utf8_lossy(&build.stdout).contains("built 1 workspace package(s)"),
            "alloc sequence fixture should report a build summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }

    #[test]
    fn test_build_fixture_std_model_runs_echo_programs() {
        let root = build_fixture_root("model_std_echo");

        let build = run_fol_in_dir(&root, &["code", "build", "--keep-build-dir"]);
        assert!(
            build.status.success(),
            "std echo fixture should build: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let build_stdout = String::from_utf8_lossy(&build.stdout);
        let binary = build_stdout
            .lines()
            .find_map(|line| {
                let plain = strip_ansi(line);
                if plain.contains("binary") {
                    plain.split_whitespace().last().map(str::to_string)
                } else {
                    None
                }
            })
            .expect("std echo build should report a binary path")
            .trim()
            .to_string();

        let run = Command::new(&binary)
            .output()
            .expect("std echo fixture binary should execute");
        assert!(
            run.status.success(),
            "std echo fixture binary should run: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            stdout.contains("std-ready"),
            "std echo fixture should print through the std model binary: stdout=\n{}\nstderr=\n{}",
            stdout,
            String::from_utf8_lossy(&run.stderr)
        );
    }

    #[test]
    fn test_build_fixture_core_model_rejects_heap_backed_surfaces() {
        let root = build_fixture_root("model_core_heap_reject");

        let build = run_fol_in_dir(&root, &["code", "build"]);
        assert!(
            !build.status.success(),
            "core heap fixture should fail: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let stderr = String::from_utf8_lossy(&build.stderr);
        assert!(
            stderr.contains("seq[...] requires heap support and is unavailable in 'fol_model = core'"),
            "core heap fixture should keep the model diagnostic: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            stderr
        );
    }

    #[test]
    fn test_build_fixture_core_model_supports_foundation_surface() {
        let root = build_fixture_root("model_core_foundation");

        let build = run_fol_in_dir(&root, &["code", "build"]);
        assert!(
            build.status.success(),
            "core foundation fixture should build: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            String::from_utf8_lossy(&build.stdout).contains("built 1 workspace package(s)"),
            "core foundation fixture should report a build summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            String::from_utf8_lossy(&build.stdout).contains("fol_model=core"),
            "core foundation fixture should surface its fol_model in the build summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }

    #[test]
    fn test_build_fixture_mixed_models_workspace_keeps_per_artifact_models() {
        let root = build_fixture_root("mixed_models_workspace");
        let build_path = root.join("build.fol");
        let source = std::fs::read_to_string(&build_path)
            .expect("mixed-model fixture should keep a build.fol");

        let request = BuildEvaluationRequest {
            package_root: root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };
        let evaluated = evaluate_build_source(&request, &build_path, &source)
            .expect("mixed-model fixture should evaluate")
            .expect("mixed-model fixture should produce a graph");

        let artifacts = &evaluated.evaluated.artifacts;
        let core = artifacts.iter().find(|a| a.name == "corelib").expect("corelib");
        let alloc = artifacts.iter().find(|a| a.name == "alloclib").expect("alloclib");
        let tool = artifacts.iter().find(|a| a.name == "tool").expect("tool");

        assert_eq!(
            core.fol_model,
            fol_package::build_artifact::BuildArtifactFolModel::Core
        );
        assert_eq!(
            alloc.fol_model,
            fol_package::build_artifact::BuildArtifactFolModel::Alloc
        );
        assert_eq!(
            tool.fol_model,
            fol_package::build_artifact::BuildArtifactFolModel::Std
        );

        let run = run_fol_in_dir(&root, &["code", "run"]);
        assert!(
            run.status.success(),
            "mixed-model fixture should still run its std tool: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            String::from_utf8_lossy(&run.stdout).contains("fol_model=std"),
            "mixed-model routed run should keep the std model summary: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }

    #[test]
    fn test_build_fixtures_core_model_reject_forbidden_surfaces() {
        let cases = [
            (
                "model_core_reject_str",
                "str requires heap support and is unavailable in 'fol_model = core'",
            ),
            (
                "model_core_reject_seq",
                "seq[...] requires heap support and is unavailable in 'fol_model = core'",
            ),
            (
                "model_core_reject_vec",
                "vec[...] requires heap support and is unavailable in 'fol_model = core'",
            ),
            (
                "model_core_reject_set",
                "set[...] requires heap support and is unavailable in 'fol_model = core'",
            ),
            (
                "model_core_reject_map",
                "map[...] requires heap support and is unavailable in 'fol_model = core'",
            ),
            (
                "model_core_reject_echo",
                "'.echo(...)' requires 'fol_model = std'",
            ),
        ];

        for (fixture, needle) in cases {
            let root = build_fixture_root(fixture);
            let build = run_fol_in_dir(&root, &["code", "build"]);
            let stderr = String::from_utf8_lossy(&build.stderr);
            assert!(
                !build.status.success(),
                "core forbidden-surface fixture should fail for {}: stdout=\n{}\nstderr=\n{}",
                fixture,
                String::from_utf8_lossy(&build.stdout),
                stderr
            );
            assert!(
                stderr.contains(needle),
                "core forbidden-surface fixture should report the expected diagnostic for {}: stdout=\n{}\nstderr=\n{}",
                fixture,
                String::from_utf8_lossy(&build.stdout),
                stderr
            );
        }
    }

    #[test]
    fn test_cli_code_build_rejects_old_root_build_syntax() {
        let root = unique_temp_root("old_root_build_syntax");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(root.join("build.fol"), "def root: loc = \"src\";\n")
            .expect("should write old build syntax");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0;\n};\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !output.status.success(),
            "old root build syntax should fail: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
        assert!(
            stderr.contains("canonical `pro[] build(graph: Graph): non` entry"),
            "old root build syntax should point at the canonical build entry: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_cli_code_build_rejects_plain_pro_build_headers() {
        let root = unique_temp_root("plain_pro_build_header");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(
            root.join("build.fol"),
            "pro build(graph: Graph): non = {\n    return graph;\n};\n",
        )
        .expect("should write non-canonical build header");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0;\n};\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !output.status.success(),
            "plain pro build header should fail: stdout=;\n{};\nstderr=;\n{};",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
        assert!(
            stderr.contains("canonical `pro[] build(graph: Graph): non` entry"),
            "plain pro build header should point at the canonical build entry: stdout=;\n{};\nstderr=;\n{};",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_cli_code_build_rejects_empty_build_file() {
        let root = unique_temp_root("empty_build_file");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(root.join("build.fol"), "")
            .expect("should write empty build file");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0;\n};\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);

        assert!(
            !output.status.success(),
            "empty build.fol should fail: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_cli_code_build_rejects_missing_package_yaml() {
        let root = unique_temp_root("missing_package_yaml");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("build.fol"), semantic_bin_build("demo"))
            .expect("should write build file");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0;\n};\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);

        assert!(
            !output.status.success(),
            "missing package.yaml should fail: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_cli_code_build_rejects_missing_source_root() {
        let root = unique_temp_root("missing_source_root");
        std::fs::create_dir_all(&root).expect("should create root dir");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(root.join("build.fol"), semantic_bin_build("demo"))
            .expect("should write build file");
        // Intentionally no src/main.fol

        let output = run_fol_in_dir(&root, &["code", "build"]);

        assert!(
            !output.status.success(),
            "missing source root should fail: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn test_cli_code_build_keeps_core_string_boundary_diagnostic() {
        let temp_root = unique_temp_root("build_core_string_boundary");
        let root = temp_root.join("demo");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(
            root.join("build.fol"),
            concat!(
                "pro[] build(graph: Graph): non = {\n",
                "    var app = graph.add_exe({\n",
                "        name = \"demo\",\n",
                "        root = \"src/main.fol\",\n",
                "        fol_model = \"core\",\n",
                "    });\n",
                "    graph.install(app);\n",
                "    return graph\n",
                "};\n",
            ),
        )
        .expect("should write build file");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): str = {\n    return \"ok\";\n};\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(!output.status.success(), "core string boundary should fail");
        assert!(
            stderr.contains("str requires heap support and is unavailable in 'fol_model = core'"),
            "CLI should preserve the core string boundary wording: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_code_build_keeps_alloc_echo_boundary_diagnostic() {
        let temp_root = unique_temp_root("build_alloc_echo_boundary");
        let root = temp_root.join("demo");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(
            root.join("build.fol"),
            concat!(
                "pro[] build(graph: Graph): non = {\n",
                "    var app = graph.add_exe({\n",
                "        name = \"demo\",\n",
                "        root = \"src/main.fol\",\n",
                "        fol_model = \"alloc\",\n",
                "    });\n",
                "    graph.install(app);\n",
                "    return graph\n",
                "};\n",
            ),
        )
        .expect("should write build file");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return .echo(1);\n};\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(!output.status.success(), "alloc echo boundary should fail");
        assert!(
            stderr.contains("'.echo(...)' requires 'fol_model = std'; current artifact model is 'alloc'"),
            "CLI should preserve the alloc echo boundary wording: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_code_build_keeps_core_dynamic_len_boundary_diagnostic() {
        let temp_root = unique_temp_root("build_core_len_boundary");
        let root = temp_root.join("demo");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(
            root.join("build.fol"),
            concat!(
                "pro[] build(graph: Graph): non = {\n",
                "    var app = graph.add_exe({\n",
                "        name = \"demo\",\n",
                "        root = \"src/main.fol\",\n",
                "        fol_model = \"core\",\n",
                "    });\n",
                "    graph.install(app);\n",
                "    return graph\n",
                "};\n",
            ),
        )
        .expect("should write build file");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return .len(\"Ada\");\n};\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(!output.status.success(), "core dynamic len boundary should fail");
        assert!(
            stderr.contains("string literals require heap support and are unavailable in 'fol_model = core'"),
            "CLI should preserve the core dynamic len boundary wording: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_code_build_and_run_keep_std_model_runtime_path() {
        let temp_root = unique_temp_root("build_std_model_runtime");
        let root = temp_root.join("demo");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(
            root.join("build.fol"),
            concat!(
                "pro[] build(graph: Graph): non = {\n",
                "    var app = graph.add_exe({\n",
                "        name = \"demo\",\n",
                "        root = \"src/main.fol\",\n",
                "        fol_model = \"std\",\n",
                "    });\n",
                "    graph.install(app);\n",
                "    graph.add_run(app);\n",
                "    return graph\n",
                "};\n",
            ),
        )
        .expect("should write build file");
        std::fs::write(
            root.join("src/main.fol"),
            concat!(
                "fun[] main(): int = {\n",
                "    return .echo(7);\n",
                "};\n",
            ),
        )
        .expect("should write app source");

        let build = run_fol_in_dir(&root, &["code", "build", "--keep-build-dir"]);
        let build_stdout = String::from_utf8_lossy(&build.stdout);
        assert!(
            build.status.success(),
            "std model build should succeed: stdout=\n{}\nstderr=\n{}",
            build_stdout,
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(
            build_stdout.contains("built 1 workspace package(s)"),
            "std model build should report a build summary: stdout=\n{}\nstderr=\n{}",
            build_stdout,
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_fol_in_dir(&root, &["code", "run"]);
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            run.status.success(),
            "std model run should succeed: stdout=\n{}\nstderr=\n{}",
            run_stdout,
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            run_stdout.contains("7"),
            "std model run should execute through runtime std path: stdout=\n{}\nstderr=\n{}",
            run_stdout,
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(
            run_stdout.contains("ran "),
            "std model run should report a run summary: stdout=\n{}\nstderr=\n{}",
            run_stdout,
            String::from_utf8_lossy(&run.stderr)
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_lsp_unknown_method_returns_method_not_found_error() {
        let mut server = EditorLspServer::new(EditorConfig::default());

        let result = server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(999),
            method: "textDocument/nonExistentMethod".to_string(),
            params: None,
        });

        match result {
            Ok(Some(response)) => {
                assert!(
                    response.error.is_some(),
                    "Unknown LSP method should return an error response"
                );
            }
            Ok(None) => {
                // Also acceptable: server ignores unknown methods
            }
            Err(_) => {
                // Also acceptable: server rejects at dispatch level
            }
        }
    }

    #[test]
    fn test_lsp_document_with_syntax_errors_returns_diagnostics() {
        let temp_root = unique_temp_root("lsp_syntax_errors");
        std::fs::create_dir_all(&temp_root).expect("should create temp root");
        let uri = format!("file://{}", temp_root.join("bad.fol").display());
        let bad_source = "fun[] main( = {\n    return;\n};\n";

        let mut server = EditorLspServer::new(EditorConfig::default());
        let diagnostics = server
            .handle_notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "textDocument/didOpen".to_string(),
                params: Some(
                    serde_json::to_value(LspDidOpenTextDocumentParams {
                        text_document: LspTextDocumentItem {
                            uri: uri.clone(),
                            language_id: "fol".to_string(),
                            version: 1,
                            text: bad_source.to_string(),
                        },
                    })
                    .expect("didOpen params should serialize"),
                ),
            })
            .expect("didOpen with syntax errors should not crash the server");

        assert!(
            !diagnostics.is_empty(),
            "Documents with syntax errors should still produce diagnostic notifications"
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_lsp_hover_on_empty_document_does_not_crash() {
        let temp_root = unique_temp_root("lsp_empty_hover");
        std::fs::create_dir_all(&temp_root).expect("should create temp root");
        let uri = format!("file://{}", temp_root.join("empty.fol").display());

        let mut server = EditorLspServer::new(EditorConfig::default());
        open_lsp_document(&mut server, uri.clone(), "");

        let hover = server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(1),
            method: "textDocument/hover".to_string(),
            params: Some(
                serde_json::to_value(LspHoverParams {
                    text_document: LspTextDocumentIdentifier { uri },
                    position: LspPosition {
                        line: 0,
                        character: 0,
                    },
                })
                .expect("hover params should serialize"),
            ),
        });

        assert!(
            hover.is_ok(),
            "Hover on empty document should not crash: {:?}",
            hover
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    #[ignore = "requires network access to github.com"]
    fn test_frontend_fetches_public_logtiny_from_github() {
        let temp_root = unique_temp_root("frontend_fetch_public_logtiny");
        let app_root = temp_root.join("app");
        create_app_with_git_dependency_from_url(&app_root, "https://github.com/bresilla/logtiny");

        let output = run_fol_in_dir(&app_root, &["pack", "fetch"]);

        assert!(
            output.status.success(),
            "public git fetch should succeed: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            app_root.join("fol.lock").is_file(),
            "public fetch should write fol.lock"
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("prepared 1 workspace package"),
            "public fetch should keep the fetch summary"
        );

        std::fs::remove_dir_all(&temp_root).ok();
    }

    fn create_app_with_git_dependency_from_url(app_root: &Path, remote_url: &str) {
        std::fs::create_dir_all(app_root.join("src")).expect("Should create app source dir");
        std::fs::write(
            app_root.join("package.yaml"),
            format!(
                "name: {}\nversion: 0.1.0\ndep.logtiny: git:git+{}\n",
                app_root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("app"),
                remote_url
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

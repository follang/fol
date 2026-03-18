use super::*;

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
    fn test_lsp_covers_build_fol_symbols_and_completion() {
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
            "}\n",
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

        let completion = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: JsonRpcId::Number(2),
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
    fn test_cli_code_build_rejects_old_root_build_syntax() {
        let root = unique_temp_root("old_root_build_syntax");
        std::fs::create_dir_all(root.join("src")).expect("should create source root");
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n")
            .expect("should write package metadata");
        std::fs::write(root.join("build.fol"), "def root: loc = \"src\";\n")
            .expect("should write old build syntax");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
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
            "pro build(graph: Graph): non = {\n    return graph\n}\n",
        )
        .expect("should write non-canonical build header");
        std::fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .expect("should write app source");

        let output = run_fol_in_dir(&root, &["code", "build"]);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !output.status.success(),
            "plain pro build header should fail: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
        assert!(
            stderr.contains("canonical `pro[] build(graph: Graph): non` entry"),
            "plain pro build header should point at the canonical build entry: stdout=\n{}\nstderr=\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );

        std::fs::remove_dir_all(&root).ok();
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
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .expect("Should write app source");
    }

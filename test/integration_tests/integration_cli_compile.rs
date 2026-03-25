use super::*;

    #[test]
    fn test_cli_single_file_compile_succeeds_with_package_parser() {
        let output = run_fol(&["test/parser/simple_var.fol"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept declaration-only single-file input, got status {:?} and output:\n{}",
            output.status.code(),
            stdout
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile"
        );
    }

    #[test]
    fn test_cli_typecheck_accepts_loc_imported_symbols_after_workspace_handoff() {
        use std::fs;

        let temp_root = unique_temp_root("cli_loc_import");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create the shared fixture directory");
        fs::create_dir_all(&app_root).expect("Should create the app fixture directory");
        fs::write(shared_root.join("lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the shared export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer;\n};\n",
        )
        .expect("Should write the loc import fixture");

        let output = run_fol(&[app_root
            .to_str()
            .expect("Temporary app fixture path should be valid UTF-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should typecheck imported loc symbols through the full workspace-aware chain, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile for loc-imported packages"
        );
    }

    #[test]
    fn test_cli_resolves_std_imports_with_explicit_std_root_configuration() {
        use std::fs;

        let temp_root = unique_temp_root("cli_std_root_import");
        let std_root = temp_root.join("std");
        let app_root = temp_root.join("app");
        fs::create_dir_all(std_root.join("fmt"))
            .expect("Should create the standard-library fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(
            std_root.join("fmt/value.fol"),
            "var[exp] answer: int = 42;\n",
        )
        .expect("Should write the standard-library export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use fmt: std = {fmt};\nfun[] main(): int = {\n    return answer;\n};\n",
        )
        .expect("Should write the std import fixture");

        let output = run_fol(&[
            "--std-root",
            std_root
                .to_str()
                .expect("Temporary std-root fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should resolve std imports through an explicit std-root flag, got status {:?} and output:;\n{};",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile for std-imported packages",
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_accepts_explicit_package_store_root_configuration() {
        use std::fs;

        let temp_root = unique_temp_root("cli_package_store_root");
        let store_root = temp_root.join("store");
        let app_root = temp_root.join("app");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create the package-store fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the installed package metadata fixture");
        fs::create_dir_all(store_root.join("json/src"))
            .expect("Should create the installed package export root fixture");
        fs::write(
            store_root.join("json/build.fol"),
            "pro[] build(): non = {\n    return;\n};\n",
        )
        .expect("Should write the installed package build fixture");
        fs::write(
            store_root.join("json/src/lib.fol"),
            "var[exp] answer: int = 42;\n",
        )
        .expect("Should write the installed package export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use json: pkg = {json};\nfun[] main(): int = {\n    return json::src::answer;\n};\n",
        )
        .expect("Should write the pkg import fixture");

        let output = run_fol(&[
            "--package-store-root",
            store_root
                .to_str()
                .expect("Temporary package-store fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept an explicit package-store root and resolve pkg imports, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile for pkg-imported packages"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_loc_import_graphs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_loc");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create the shared fixture directory");
        fs::create_dir_all(&app_root).expect("Should create the app fixture directory");
        fs::write(shared_root.join("lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the shared export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer;\n};\n",
        )
        .expect("Should write the loc import fixture");

        let output = run_fol(&[
            "--dump-lowered",
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for loc-import graphs, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(stdout.contains("workspace entry=app"));
        assert!(stdout.contains("package app"));
        assert!(stdout.contains("package shared"));
        assert!(stdout.contains("entry-candidates"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_std_import_graphs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_std");
        let std_root = temp_root.join("std");
        let app_root = temp_root.join("app");
        fs::create_dir_all(std_root.join("fmt"))
            .expect("Should create the standard-library fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(
            std_root.join("fmt/value.fol"),
            "var[exp] answer: int = 42;\n",
        )
        .expect("Should write the standard-library export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use fmt: std = {fmt};\nfun[] main(): int = {\n    return answer;\n};\n",
        )
        .expect("Should write the std import fixture");

        let output = run_fol(&[
            "--dump-lowered",
            "--std-root",
            std_root
                .to_str()
                .expect("Temporary std-root fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for std-import graphs, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(stdout.contains("workspace entry=app"));
        assert!(stdout.contains("package app"));
        assert!(stdout.contains("package fmt"));
        assert!(stdout.contains("entry-candidates"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_pkg_import_graphs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_pkg");
        let store_root = temp_root.join("store");
        let app_root = temp_root.join("app");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create the package-store fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the installed package metadata fixture");
        fs::create_dir_all(store_root.join("json/src"))
            .expect("Should create the installed package export root fixture");
        fs::write(
            store_root.join("json/build.fol"),
            "pro[] build(): non = {\n    return;\n};\n",
        )
        .expect("Should write the installed package build fixture");
        fs::write(
            store_root.join("json/src/lib.fol"),
            "var[exp] answer: int = 42;\n",
        )
        .expect("Should write the installed package export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use json: pkg = {json};\nfun[] main(): int = {\n    return json::src::answer;\n};\n",
        )
        .expect("Should write the pkg import fixture");

        let output = run_fol(&[
            "--dump-lowered",
            "--package-store-root",
            store_root
                .to_str()
                .expect("Temporary package-store fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for pkg-import graphs, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(stdout.contains("workspace entry=app"));
        assert!(stdout.contains("package app"));
        assert!(stdout.contains("package json"));
        assert!(stdout.contains("entry-candidates"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_intrinsic_comparison_calls() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_intrinsic_comparisons");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic comparison fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(): bol = {\n",
                "    var same: bol = .eq(1, 1);\n",
                "    var ordered: bol = .lt(\"Ada\", \"Lin\");\n",
                "    return .ge('z', 'a');\n",
                "};\n",
            ),
        )
        .expect("Should write intrinsic comparison fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("Temporary intrinsic comparison fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for intrinsic comparison calls, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.matches("IntrinsicCall").count() >= 3,
            "Lowered dump should retain explicit intrinsic calls for comparison families, got:\n{}",
            stdout,
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_intrinsic_comparison_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_intrinsic_comparison_failures");
        fs::create_dir_all(&temp_root)
            .expect("Should create temp intrinsic comparison failure fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(): bol = {\n",
                "    return .lt(true, false);\n",
                "};\n",
            ),
        )
        .expect("Should write intrinsic comparison failure fixture");

        let output = run_fol(&[
            "--json",
            fixture.to_str().expect(
                "Temporary intrinsic comparison failure fixture path should be valid UTF-8",
            ),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for invalid intrinsic comparison calls",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");
        let ordered_error = diagnostics.iter().find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .map(|message| message.contains(".lt(...) expects two ordered scalar operands"))
                .unwrap_or(false)
        });

        assert!(
            ordered_error.is_some(),
            "Expected intrinsic comparison diagnostic in CLI JSON output, got: {json}"
        );
        assert!(
            ordered_error
                .and_then(|diagnostic| diagnostic["location"].as_object())
                .is_some(),
            "Expected intrinsic comparison diagnostic to keep a structured location, got: {json}"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_intrinsic_boolean_calls() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_intrinsic_boolean");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic boolean fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(flag: bol): bol = {\n",
                "    var inverted: bol = .not(flag);\n",
                "    return .not(inverted);\n",
                "};\n",
            ),
        )
        .expect("Should write intrinsic boolean fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("Temporary intrinsic boolean fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for intrinsic boolean calls, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.matches("IntrinsicCall").count() >= 2,
            "Lowered dump should retain explicit intrinsic calls for '.not', got:\n{}",
            stdout,
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_intrinsic_boolean_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_intrinsic_boolean_failures");
        fs::create_dir_all(&temp_root)
            .expect("Should create temp intrinsic boolean failure fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!("fun[] main(): bol = {\n", "    return .not(1);\n", "};\n",),
        )
        .expect("Should write intrinsic boolean failure fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("Temporary intrinsic boolean failure fixture path should be valid UTF-8"),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for invalid intrinsic boolean calls",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");
        let boolean_error = diagnostics.iter().find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .map(|message| message.contains(".not(...) expects one boolean operand"))
                .unwrap_or(false)
        });

        assert!(
            boolean_error.is_some(),
            "Expected intrinsic boolean diagnostic in CLI JSON output, got: {json}"
        );
        assert!(
            boolean_error
                .and_then(|diagnostic| diagnostic["location"].as_object())
                .is_some(),
            "Expected intrinsic boolean diagnostic to keep a structured location, got: {json}"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_intrinsic_length_calls() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_intrinsic_length");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic length fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(items: seq[int]): int = {\n",
                "    var text: int = .len(\"Ada\");\n",
                "    var count: int = .len(items);\n",
                "    return count;\n",
                "};\n",
            ),
        )
        .expect("Should write intrinsic length fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("Temporary intrinsic length fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for intrinsic length calls, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.matches("LengthOf").count() >= 2,
            "Lowered dump should retain dedicated LengthOf instructions for '.len', got:\n{}",
            stdout,
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_intrinsic_length_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_intrinsic_length_failures");
        fs::create_dir_all(&temp_root)
            .expect("Should create temp intrinsic length failure fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "typ Flagged: rec = {\n",
                "    name: str;\n",
                "};\n",
                "fun[] main(value: Flagged): int = {\n",
                "    return .len(value);\n",
                "};\n",
            ),
        )
        .expect("Should write intrinsic length failure fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("Temporary intrinsic length failure fixture path should be valid UTF-8"),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for invalid intrinsic length calls",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");
        let length_error = diagnostics.iter().find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .map(|message| {
                    message.contains(
                        ".len(...) expects one string, array, vector, sequence, set, or map operand",
                    )
                })
                .unwrap_or(false)
        });

        assert!(
            length_error.is_some(),
            "Expected intrinsic length diagnostic in CLI JSON output, got: {json}"
        );
        assert!(
            length_error
                .and_then(|diagnostic| diagnostic["location"].as_object())
                .is_some(),
            "Expected intrinsic length diagnostic to keep a structured location, got: {json}"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_intrinsic_echo_calls() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_intrinsic_echo");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic echo fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(flag: bol): bol = {\n",
                "    return .echo(flag);\n",
                "};\n",
            ),
        )
        .expect("Should write intrinsic echo fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("Temporary intrinsic echo fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for intrinsic echo calls, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("RuntimeHook"),
            "Lowered dump should retain explicit runtime-hook instructions for '.echo', got:\n{}",
            stdout,
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_intrinsic_echo_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_intrinsic_echo_failures");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic echo failure fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!("fun[] main(): int = {\n", "    return .echo();\n", "};\n",),
        )
        .expect("Should write intrinsic echo failure fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("Temporary intrinsic echo failure fixture path should be valid UTF-8"),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for invalid intrinsic echo calls",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");
        let echo_error = diagnostics.iter().find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .map(|message| {
                    message.contains(".echo(...) expects exactly 1 argument(s) but got 0")
                })
                .unwrap_or(false)
        });

        assert!(
            echo_error.is_some(),
            "Expected intrinsic echo diagnostic in CLI JSON output, got: {json}"
        );
        assert!(
            echo_error
                .and_then(|diagnostic| diagnostic["location"].as_object())
                .is_some(),
            "Expected intrinsic echo diagnostic to keep a structured location, got: {json}"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_v3_intrinsic_boundaries_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_intrinsic_v3_boundaries");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic V3 boundary fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(value: int): int = {\n",
                "    return .de_alloc(value);\n",
                "};\n",
            ),
        )
        .expect("Should write intrinsic V3 boundary fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("Temporary intrinsic V3 boundary fixture path should be valid UTF-8"),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for V3-only intrinsic calls during the V1 milestone",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");
        let v3_error = diagnostics.iter().find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .map(|message| {
                    message.contains(
                        ".de_alloc(...) is planned for a future release",
                    )
                })
                .unwrap_or(false)
        });

        assert!(
            v3_error.is_some(),
            "Expected explicit V3 intrinsic boundary diagnostic in CLI JSON output, got: {json}"
        );
        assert!(
            v3_error
                .and_then(|diagnostic| diagnostic["location"].as_object())
                .is_some(),
            "Expected V3 intrinsic boundary diagnostic to keep a structured location, got: {json}"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_cast_intrinsic_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_cast_intrinsic_failures");
        fs::create_dir_all(&temp_root).expect("Should create temp cast intrinsic failure fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "var text: str = \"label\";\n",
                "var target: int = 0;\n",
                "fun[] main(value: int): int = {\n",
                "    return value as text;\n",
                "};\n",
                "fun[] side(value: int): int = {\n",
                "    return value cast target;\n",
                "};\n",
            ),
        )
        .expect("Should write cast intrinsic failure fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("Temporary cast intrinsic fixture path should be valid UTF-8"),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for deferred cast intrinsic surfaces during the V1 milestone",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");

        for expected in [
            "operator 'as' is not yet supported",
            "operator 'cast' is not yet supported",
        ] {
            let diagnostic = diagnostics.iter().find(|diagnostic| {
                diagnostic["message"]
                    .as_str()
                    .map(|message| message.contains(expected))
                    .unwrap_or(false)
            });

            assert!(
                diagnostic.is_some(),
                "Expected cast intrinsic diagnostic containing '{expected}', got: {json}"
            );
            assert!(
                diagnostic
                    .and_then(|diagnostic| diagnostic["location"].as_object())
                    .is_some(),
                "Expected cast intrinsic diagnostic '{expected}' to keep a structured location, got: {json}"
            );
        }

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_folder_compile_succeeds_with_package_parser() {
        use std::fs;

        let temp_root = unique_temp_root("cli_folder_compile");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI folder fixture");
        fs::write(temp_root.join("00_first.fol"), "var first = 1;\n")
            .expect("Should write first declaration source");
        fs::write(temp_root.join("10_second.fol"), "var second = 2;\n")
            .expect("Should write second declaration source");

        let output = run_fol(&[temp_root
            .to_str()
            .expect("CLI folder fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept declaration-only folders, got status {:?} and output:\n{}",
            output.status.code(),
            stdout
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful folder compile"
        );

        fs::remove_dir_all(&temp_root).ok();
    }


use fol_frontend::run_command_from_args_in_dir;
use std::fs;
use std::path::PathBuf;

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "fol_frontend_editor_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root should canonicalize")
}

#[test]
fn editor_lsp_command_is_publicly_dispatchable() {
    let root = repo_root();
    let (_, result) =
        run_command_from_args_in_dir(["fol", "tool", "lsp"], root.join("xtra/logtiny"))
            .expect("editor lsp should dispatch");

    assert_eq!(result.command, "lsp");
    assert!(result.summary.contains("fol tool lsp"));
    assert!(result.summary.contains("diagnostics"));
    assert!(result.summary.contains("hover"));
    assert!(result.summary.contains("definition"));
    assert!(result.summary.contains("symbols"));
    assert!(result.summary.contains("completion"));
    assert!(result
        .summary
        .contains("features=diagnostics,hover,definition,symbols,completion"));
}

#[test]
fn editor_surface_stays_under_tool_not_a_parallel_editor_group() {
    let root = repo_root();
    let error = run_command_from_args_in_dir(["fol", "editor", "lsp"], root.join("xtra/logtiny"))
        .expect_err("`fol editor` should not exist as a parallel public surface");
    let json = fol_frontend::FrontendOutput::new(fol_frontend::FrontendOutputConfig {
        mode: fol_frontend::OutputMode::Json,
    })
    .render_error(&error)
    .expect("json render should succeed");
    assert!(json.contains("\"kind\": \"FrontendInvalidInput\""));
    assert!(json.contains("unrecognized subcommand"));
    assert!(json.contains("editor"));
}

#[test]
fn editor_tool_surface_rejects_placeholder_future_commands() {
    let root = repo_root();

    for command in [
        ["fol", "tool", "references"],
        ["fol", "tool", "rename"],
        ["fol", "tool", "format"],
        ["fol", "tool", "semanticTokens"],
    ] {
        let error = run_command_from_args_in_dir(command, root.join("xtra/logtiny"))
            .expect_err("unsupported future tool command should stay off the public surface");
        let json = fol_frontend::FrontendOutput::new(fol_frontend::FrontendOutputConfig {
            mode: fol_frontend::OutputMode::Json,
        })
        .render_error(&error)
        .expect("json render should succeed");

        assert!(json.contains("\"kind\": \"FrontendInvalidInput\""));
        assert!(json.contains("unrecognized subcommand"));
        assert!(json.contains(command[2]));
    }
}

#[test]
fn editor_file_commands_dispatch_against_real_fol_fixtures() {
    let root = repo_root();
    let fixture = "test/apps/fixtures/record_flow/main.fol";

    let (_, parse) = run_command_from_args_in_dir(["fol", "tool", "parse", fixture], &root)
        .expect("editor parse should dispatch");
    let (_, highlight) = run_command_from_args_in_dir(["fol", "tool", "highlight", fixture], &root)
        .expect("editor highlight should dispatch");
    let (_, symbols) = run_command_from_args_in_dir(["fol", "tool", "symbols", fixture], &root)
        .expect("editor symbols should dispatch");

    assert_eq!(parse.command, "parse");
    assert!(parse.summary.contains("grammar_bytes="));
    assert_eq!(highlight.command, "highlight");
    assert!(highlight.summary.contains("capture_count="));
    assert!(highlight.summary.contains("captures="));
    assert!(highlight.summary.contains("intrinsic_names="));
    assert_eq!(symbols.command, "symbols");
    assert!(symbols.summary.contains("query_snapshots=3"));
}

#[test]
fn editor_commands_respect_requested_output_mode() {
    let root = repo_root();
    let fixture = "test/apps/fixtures/record_flow/main.fol";
    let (output, result) = run_command_from_args_in_dir(
        ["fol", "tool", "--output", "plain", "parse", fixture],
        &root,
    )
    .expect("editor parse should support output mode");
    let rendered = output
        .render_command_summary(&result)
        .expect("plain output should render");

    assert!(rendered.contains("command: parse"));
    assert!(rendered.contains("summary: loaded"));
    assert!(rendered.contains("bytes="));
}

#[test]
fn editor_commands_do_not_require_workspace_discovery() {
    let root = temp_root("no_workspace");
    fs::create_dir_all(&root).expect("should create temp root");
    let file = root.join("sample.fol");
    fs::write(&file, "fun[] main(): int = {\n    return 0\n}\n")
        .expect("should write sample source");

    let (_, result) = run_command_from_args_in_dir(
        ["fol", "tool", "parse", file.to_string_lossy().as_ref()],
        &root,
    )
    .expect("editor parse should not need a workspace root");

    assert_eq!(result.command, "parse");
    assert!(result.summary.contains("path="));

    fs::remove_dir_all(root).ok();
}

#[test]
fn editor_command_plain_output_stays_snapshot_stable_for_real_fixtures() {
    let root = repo_root();
    let fixture = "xtra/logtiny/src/log.fol";
    let (output, result) = run_command_from_args_in_dir(
        ["fol", "tool", "--output", "plain", "symbols", fixture],
        &root,
    )
    .expect("editor symbols should support plain output");
    let rendered = output
        .render_command_summary(&result)
        .expect("plain output should render");

    assert_eq!(
        rendered,
        "command: symbols\nsummary: symbol query ready with 803 bytes (lines=52, path=xtra/logtiny/src/log.fol, query_snapshots=3, symbol_candidates=8)"
    );
}

#[test]
fn editor_command_json_errors_keep_stable_shapes() {
    let error = run_command_from_args_in_dir(
        [
            "fol",
            "tool",
            "--output",
            "json",
            "parse",
            "missing-editor-file.fol",
        ],
        repo_root(),
    )
    .expect_err("missing file should fail");
    let rendered = fol_frontend::FrontendOutput::new(fol_frontend::FrontendOutputConfig {
        mode: fol_frontend::OutputMode::Json,
    })
    .render_error(&error)
    .expect("json render should succeed");
    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("stderr should be json");
    assert_eq!(parsed["kind"], "FrontendCommandFailed");
    assert!(parsed["message"]
        .as_str()
        .expect("message should be a string")
        .contains("failed to read"));
    assert!(parsed["notes"].is_array());
}

#[test]
fn editor_lsp_reports_workspace_guidance_when_no_root_is_present() {
    let root = temp_root("missing_lsp_root");
    fs::create_dir_all(&root).expect("should create temp root");
    let error = run_command_from_args_in_dir(["fol", "tool", "lsp"], &root)
        .expect_err("editor lsp should require a discovered root");
    let rendered = fol_frontend::FrontendOutput::new(fol_frontend::FrontendOutputConfig {
        mode: fol_frontend::OutputMode::Json,
    })
    .render_error(&error)
    .expect("json render should succeed");
    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("stderr should be json");

    assert_eq!(parsed["kind"], "FrontendWorkspaceNotFound");
    let notes = parsed["notes"]
        .as_array()
        .expect("notes should be an array");
    assert!(notes.iter().any(|note| note
        .as_str()
        .unwrap_or("")
        .contains("start the editor inside a FOL package or workspace root")));

    fs::remove_dir_all(root).ok();
}

#[test]
fn tree_generate_command_writes_bundle_layout() {
    let root = temp_root("tree_generate");
    let output = root.join("bundle");

    let (_, result) = run_command_from_args_in_dir(
        [
            "fol",
            "tool",
            "tree",
            "generate",
            output.to_string_lossy().as_ref(),
        ],
        repo_root(),
    )
    .expect("tree generate should dispatch");

    assert_eq!(result.command, "tree generate");
    assert!(output.join("grammar.js").is_file());
    assert!(output.join("queries/fol/highlights.scm").is_file());
    assert!(output.join("queries/fol/locals.scm").is_file());
    assert!(output.join("queries/fol/symbols.scm").is_file());
    assert!(output.join("test/corpus/declarations.txt").is_file());

    fs::remove_dir_all(root).ok();
}

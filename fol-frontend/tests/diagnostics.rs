use fol_frontend::{FrontendOutput, FrontendOutputConfig, OutputMode, run_command_from_args_in_dir};
use std::fs;
use std::path::PathBuf;

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "fol_frontend_diagnostics_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ))
}

#[test]
fn frontend_workspace_discovery_failures_render_consistently_across_output_modes() {
    let root = temp_root("missing_root");
    fs::create_dir_all(&root).expect("should create empty root");

    let error = run_command_from_args_in_dir(["fol", "work", "info"], &root).unwrap_err();

    let human = FrontendOutput::new(FrontendOutputConfig::default())
        .render_error(&error)
        .expect("human render should succeed");
    let plain = FrontendOutput::new(FrontendOutputConfig {
        mode: OutputMode::Plain,
        ..FrontendOutputConfig::default()
    })
    .render_error(&error)
    .expect("plain render should succeed");
    let json = FrontendOutput::new(FrontendOutputConfig {
        mode: OutputMode::Json,
        ..FrontendOutputConfig::default()
    })
    .render_error(&error)
    .expect("json render should succeed");

    assert!(human.contains("FrontendWorkspaceNotFound"));
    assert!(human.contains("fol init --bin"));
    assert!(plain.contains("note: run `fol init --workspace`"));
    assert!(json.contains("\"kind\": \"FrontendWorkspaceNotFound\""));
    assert!(json.contains("\"notes\": ["));

    fs::remove_dir_all(root).ok();
}

#[test]
fn frontend_parse_failures_keep_structured_help_notes() {
    let error = run_command_from_args_in_dir(["fol", "emit", "wat"], std::env::temp_dir())
        .unwrap_err();
    let json = FrontendOutput::new(FrontendOutputConfig {
        mode: OutputMode::Json,
        ..FrontendOutputConfig::default()
    })
    .render_error(&error)
    .expect("json render should succeed");

    assert!(error.message().contains("invalid value"));
    assert!(json.contains("fol --help"));
}

use fol_frontend::{FrontendCli, FrontendOutput, FrontendOutputConfig};

#[test]
fn help_output_keeps_commands_and_aliases_stable() {
    let help = FrontendCli::root_help_text();

    assert!(help.contains("work"));
    assert!(help.contains("pack"));
    assert!(help.contains("code"));
    assert!(help.contains("tool"));
    assert!(help.contains("[aliases: w]"));
    assert!(help.contains("[aliases: p]"));
    assert!(help.contains("[aliases: c]"));
    assert!(help.contains("[aliases: t]"));
}

#[test]
fn help_output_contains_usage_and_options() {
    let help = FrontendCli::root_help_text();

    assert!(help.contains("Usage:"));
    assert!(help.contains("Commands:"));
    assert!(help.contains("Options:"));
    assert!(help.contains("--help"));
    assert!(help.contains("--version"));
}

#[test]
fn human_output_uses_colored_status_blocks() {
    let output = FrontendOutput::new(FrontendOutputConfig::default());

    let rendered = output.render_human_status("Built", "target/demo/bin");

    assert!(rendered.contains("\u{1b}["));
    assert!(rendered.contains("Built"));
    assert!(rendered.contains("target/demo/bin"));
}

use fol_frontend::{FrontendCli, FrontendOutput, FrontendOutputConfig};

#[test]
fn help_output_keeps_sections_examples_and_aliases_stable() {
    let help = FrontendCli::command().render_long_help().to_string();

    assert!(help.contains("Workflow Commands:"));
    assert!(help.contains("Workspace Commands:"));
    assert!(help.contains("Shell Commands:"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("fol emit rust"));
    assert!(help.contains("make"));
    assert!(help.contains("sync"));
    assert!(help.contains("purge"));
}

#[test]
fn help_output_keeps_grouped_sections_in_stable_order() {
    let help = FrontendCli::command().render_long_help().to_string();

    let workflow = help.find("Workflow Commands:").expect("workflow section");
    let workspace = help.find("Workspace Commands:").expect("workspace section");
    let shell = help.find("Shell Commands:").expect("shell section");
    let examples = help.find("Examples:").expect("examples section");

    assert!(workflow < workspace);
    assert!(workspace < shell);
    assert!(shell < examples);
}

#[test]
fn human_output_uses_colored_status_blocks() {
    let output = FrontendOutput::new(FrontendOutputConfig::default());

    let rendered = output.render_human_status("Built", "target/demo/bin");

    assert!(rendered.contains("\u{1b}["));
    assert!(rendered.contains("Built"));
    assert!(rendered.contains("target/demo/bin"));
}

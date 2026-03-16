use fol_frontend::FrontendCli;

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

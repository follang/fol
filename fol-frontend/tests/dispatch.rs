use fol_frontend::run_from_args_with_io;

#[test]
fn frontend_run_from_args_writes_rendered_command_summaries_without_root_duplication() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run_from_args_with_io(["fol", "_complete", "bu"], &mut stdout, &mut stderr);

    assert_eq!(code, 0);
    assert!(String::from_utf8(stdout).unwrap().contains("build"));
    assert!(stderr.is_empty());
}

#[test]
fn frontend_run_from_args_writes_rendered_errors_without_root_duplication() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run_from_args_with_io(["fol", "emit", "wat"], &mut stdout, &mut stderr);

    assert_eq!(code, 1);
    assert!(stdout.is_empty());
    assert!(String::from_utf8(stderr).unwrap().contains("fol --help"));
}

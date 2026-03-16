fn main() {
    std::process::exit(fol_frontend::run_from_args(std::env::args_os()));
}

#[test]
fn root_binary_is_now_only_a_frontend_shim() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = fol_frontend::run_from_args_with_io(["fol", "--help"], &mut stdout, &mut stderr);

    assert_eq!(code, 0);
    assert!(String::from_utf8(stdout)
        .expect("help should be utf8")
        .contains("User-facing frontend for the FOL toolchain"));
    assert!(stderr.is_empty());
}

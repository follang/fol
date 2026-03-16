use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_apps_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

fn fixture_root(name: &str) -> PathBuf {
    Path::new("test/apps/fixtures").join(name)
}

fn run_fol(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fol"))
        .args(args)
        .output()
        .expect("should run fol CLI")
}

fn compile_app(entry: &Path) -> std::process::Output {
    run_fol(&[entry.to_str().expect("fixture path should be valid utf-8")])
}

fn compile_app_keep_build_dir(entry: &Path) -> std::process::Output {
    run_fol(&[
        "--keep-build-dir",
        entry.to_str().expect("fixture path should be valid utf-8"),
    ])
}

fn compile_app_with_roots(
    entry: &Path,
    std_root: Option<&Path>,
    package_store_root: Option<&Path>,
    keep_build_dir: bool,
) -> std::process::Output {
    let mut args = Vec::new();
    if keep_build_dir {
        args.push("--keep-build-dir".to_string());
    }
    if let Some(std_root) = std_root {
        args.push("--std-root".to_string());
        args.push(
            std_root
                .to_str()
                .expect("std root should be valid utf-8")
                .to_string(),
        );
    }
    if let Some(package_store_root) = package_store_root {
        args.push("--package-store-root".to_string());
        args.push(
            package_store_root
                .to_str()
                .expect("package store root should be valid utf-8")
                .to_string(),
        );
    }
    args.push(
        entry
            .to_str()
            .expect("fixture path should be valid utf-8")
            .to_string(),
    )
    ;
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_fol(&arg_refs)
}

fn compile_app_expect_success(entry: &Path) -> std::process::Output {
    let output = compile_app(entry);
    assert!(
        output.status.success(),
        "expected app compile success\nstdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn compile_app_keep_build_dir_expect_success(entry: &Path) -> std::process::Output {
    let output = compile_app_keep_build_dir(entry);
    assert!(
        output.status.success(),
        "expected kept-build-dir app compile success\nstdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn compile_app_expect_failure(entry: &Path) -> std::process::Output {
    let output = compile_app(entry);
    assert!(
        !output.status.success(),
        "expected app compile failure\nstdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn compile_app_with_roots_expect_success(
    entry: &Path,
    std_root: Option<&Path>,
    package_store_root: Option<&Path>,
) -> std::process::Output {
    let output = compile_app_with_roots(entry, std_root, package_store_root, false);
    assert!(
        output.status.success(),
        "expected rooted app compile success\nstdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn compile_app_with_roots_keep_build_dir_expect_success(
    entry: &Path,
    std_root: Option<&Path>,
    package_store_root: Option<&Path>,
) -> std::process::Output {
    let output = compile_app_with_roots(entry, std_root, package_store_root, true);
    assert!(
        output.status.success(),
        "expected rooted kept-build-dir app compile success\nstdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn built_binary_path(output: &std::process::Output) -> PathBuf {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let marker = "binary=";
    let binary = stdout
        .lines()
        .find_map(|line| line.split(marker).nth(1))
        .expect("compile success should report a binary path")
        .trim();
    PathBuf::from(binary)
}

fn emitted_crate_root(output: &std::process::Output) -> PathBuf {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let marker = "crate_root=";
    let root = stdout
        .lines()
        .find_map(|line| line.split(marker).nth(1))
        .and_then(|tail| tail.split(" binary=").next())
        .expect("compile success should report a crate root")
        .trim();
    PathBuf::from(root)
}

fn compile_and_run_app(entry: &Path) -> std::process::Output {
    let compile_output = compile_app_expect_success(entry);
    let binary = built_binary_path(&compile_output);
    Command::new(&binary)
        .output()
        .expect("should run compiled app binary")
}

fn assert_output_contains(output: &std::process::Output, needle: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains(needle) || stderr.contains(needle),
        "expected output to contain '{}'\nstdout=\n{}\nstderr=\n{}",
        needle,
        stdout,
        stderr
    );
}

fn assert_exit_code(output: &std::process::Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "expected exit code {}\nstdout=\n{}\nstderr=\n{}",
        expected,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_artifact_paths_exist(output: &std::process::Output) {
    let root = emitted_crate_root(output);
    let binary = built_binary_path(output);
    assert!(
        root.exists(),
        "reported crate root should exist at '{}'",
        root.display()
    );
    assert!(
        binary.exists(),
        "reported binary should exist at '{}'",
        binary.display()
    );
}

#[test]
fn app_fixture_tree_exists() {
    let root = Path::new("test/apps");
    let fixtures = root.join("fixtures");

    assert!(root.exists(), "app test root should exist");
    assert!(
        fixtures.exists(),
        "app fixture root should exist at '{}'",
        fixtures.display()
    );
}

#[test]
fn app_harness_compile_only_helper_handles_success_and_failure() {
    let temp_root = unique_temp_root("compile_only");
    let good_root = temp_root.join("good");
    let bad_root = temp_root.join("bad");

    fs::create_dir_all(&good_root).expect("good root");
    fs::create_dir_all(&bad_root).expect("bad root");
    fs::write(
        good_root.join("main.fol"),
        "fun[] main(): int = {\n    return 7\n}\n",
    )
    .expect("good source");
    fs::write(
        bad_root.join("main.fol"),
        "fun[] main(): int = {\n    return \"bad\"\n}\n",
    )
    .expect("bad source");

    compile_app_expect_success(&good_root);
    compile_app_expect_failure(&bad_root);

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn app_harness_run_helper_executes_built_binary() {
    let temp_root = unique_temp_root("run_helper");
    fs::create_dir_all(&temp_root).expect("run root");
    fs::write(
        temp_root.join("main.fol"),
        "fun[] main(): int = {\n    return 5\n}\n",
    )
    .expect("run source");

    let output = compile_and_run_app(&temp_root);

    assert!(
        output.status.success(),
        "compiled app binary should run successfully"
    );

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn app_harness_root_helpers_support_std_and_pkg_layouts() {
    let temp_root = unique_temp_root("root_helpers");
    let app_root = temp_root.join("app");
    let std_root = temp_root.join("std");
    let pkg_root = temp_root.join("pkg");
    let math_root = pkg_root.join("math");

    fs::create_dir_all(&app_root).expect("app root");
    fs::create_dir_all(std_root.join("fmt")).expect("std root");
    fs::create_dir_all(math_root.join("src")).expect("pkg src");

    fs::write(
        app_root.join("main.fol"),
        concat!(
            "use fmt: std = {\"fmt\"};\n",
            "use math: pkg = {math};\n",
            "fun[] main(): int = {\n",
            "    return std_answer\n",
            "}\n",
        ),
    )
    .expect("app source");
    fs::write(std_root.join("fmt").join("lib.fol"), "var[exp] std_answer: int = 3\n")
        .expect("std source");
    fs::write(math_root.join("package.yaml"), "name: math\nversion: 0.1.0\n")
        .expect("pkg manifest");
    fs::write(math_root.join("build.fol"), "def root: loc = \"src\"\n").expect("pkg build");
    fs::write(math_root.join("src").join("lib.fol"), "var[exp] pkg_answer: int = 4\n")
        .expect("pkg source");

    compile_app_with_roots_expect_success(&app_root, Some(&std_root), Some(&pkg_root));

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn app_harness_assertion_helpers_cover_artifacts_and_status() {
    let temp_root = unique_temp_root("assertions");
    let ok_root = temp_root.join("ok");
    let bad_root = temp_root.join("bad");

    fs::create_dir_all(&ok_root).expect("ok root");
    fs::create_dir_all(&bad_root).expect("bad root");
    fs::write(
        ok_root.join("main.fol"),
        "fun[] main(): int = {\n    return 0\n}\n",
    )
    .expect("ok source");
    fs::write(
        bad_root.join("main.fol"),
        "fun[] main(): int = {\n    return unknown_name\n}\n",
    )
    .expect("bad source");

    let compile_output = compile_app_keep_build_dir_expect_success(&ok_root);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&ok_root);
    assert_exit_code(&run_output, 0);

    let failure_output = compile_app_expect_failure(&bad_root);
    assert_output_contains(&failure_output, "ResolverUnresolvedName");

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn scalar_entry_fixture_compiles_and_runs() {
    let fixture = fixture_root("scalar_entry");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn bindings_and_calls_fixture_compiles_and_runs() {
    let fixture = fixture_root("bindings_and_calls");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn control_when_fixture_compiles_and_runs() {
    let fixture = fixture_root("control_when");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn control_loop_break_fixture_compiles_and_runs() {
    let fixture = fixture_root("control_loop_break");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn same_folder_shared_scope_fixture_compiles_and_runs() {
    let fixture = fixture_root("same_folder_shared_scope");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn same_folder_hidden_visibility_fixture_fails_cleanly() {
    let fixture = fixture_root("same_folder_hidden_visibility");
    let output = compile_app_expect_failure(&fixture);

    assert_output_contains(&output, "ResolverUnresolvedName");
}

#[test]
fn subfolder_namespace_fixture_compiles_and_runs() {
    let fixture = fixture_root("subfolder_namespace");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn deep_namespace_chain_fixture_compiles_and_runs() {
    let fixture = fixture_root("deep_namespace_chain");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn loc_plain_values_fixture_compiles_and_runs() {
    let fixture = fixture_root("loc_plain_values").join("app");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn loc_types_and_records_fixture_compiles_and_runs() {
    let fixture = fixture_root("loc_types_and_records").join("app");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn loc_methods_fixture_compiles_and_runs() {
    let fixture = fixture_root("loc_methods").join("app");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn loc_recoverable_calls_fixture_compiles_and_runs() {
    let fixture = fixture_root("loc_recoverable_calls").join("app");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn std_basic_import_fixture_compiles_and_runs() {
    let root = fixture_root("std_basic_import");
    let app_root = root.join("app");
    let std_root = root.join("std");

    let compile_output =
        compile_app_with_roots_keep_build_dir_expect_success(&app_root, Some(&std_root), None);
    assert_artifact_paths_exist(&compile_output);

    let binary = built_binary_path(&compile_output);
    let run_output = Command::new(&binary)
        .output()
        .expect("should run compiled std fixture");
    assert_exit_code(&run_output, 0);
}

#[test]
fn std_namespace_import_fixture_compiles_and_runs() {
    let root = fixture_root("std_namespace_import");
    let app_root = root.join("app");
    let std_root = root.join("std");

    let compile_output =
        compile_app_with_roots_keep_build_dir_expect_success(&app_root, Some(&std_root), None);
    assert_artifact_paths_exist(&compile_output);

    let binary = built_binary_path(&compile_output);
    let run_output = Command::new(&binary)
        .output()
        .expect("should run compiled std namespace fixture");
    assert_exit_code(&run_output, 0);
}

#[test]
fn pkg_basic_import_fixture_compiles_and_runs() {
    let root = fixture_root("pkg_basic_import");
    let app_root = root.join("app");
    let pkg_root = root.join("pkg");

    let compile_output =
        compile_app_with_roots_keep_build_dir_expect_success(&app_root, None, Some(&pkg_root));
    assert_artifact_paths_exist(&compile_output);

    let binary = built_binary_path(&compile_output);
    let run_output = Command::new(&binary)
        .output()
        .expect("should run compiled pkg fixture");
    assert_exit_code(&run_output, 0);
}

#[test]
fn pkg_transitive_import_fixture_compiles_and_runs() {
    let root = fixture_root("pkg_transitive_import");
    let app_root = root.join("app");
    let pkg_root = root.join("pkg");

    let compile_output =
        compile_app_with_roots_keep_build_dir_expect_success(&app_root, None, Some(&pkg_root));
    assert_artifact_paths_exist(&compile_output);

    let binary = built_binary_path(&compile_output);
    let run_output = Command::new(&binary)
        .output()
        .expect("should run compiled transitive pkg fixture");
    assert_exit_code(&run_output, 0);
}

#[test]
fn mixed_loc_std_pkg_fixture_compiles_and_runs() {
    let root = fixture_root("mixed_loc_std_pkg");
    let app_root = root.join("app");
    let std_root = root.join("std");
    let pkg_root = root.join("pkg");

    let compile_output = compile_app_with_roots_keep_build_dir_expect_success(
        &app_root,
        Some(&std_root),
        Some(&pkg_root),
    );
    assert_artifact_paths_exist(&compile_output);

    let binary = built_binary_path(&compile_output);
    let run_output = Command::new(&binary)
        .output()
        .expect("should run compiled mixed import fixture");
    assert_exit_code(&run_output, 0);
}

#[test]
fn record_flow_fixture_compiles_and_runs() {
    let fixture = fixture_root("record_flow");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn entry_flow_fixture_compiles_and_runs() {
    let fixture = fixture_root("entry_flow");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn alias_flow_fixture_compiles_and_runs() {
    let fixture = fixture_root("alias_flow");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn method_flow_fixture_compiles_and_runs() {
    let fixture = fixture_root("method_flow");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn container_linear_fixture_compiles_and_runs() {
    let fixture = fixture_root("container_linear");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn container_map_set_fixture_compiles_and_runs() {
    let fixture = fixture_root("container_map_set");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

#[test]
fn container_cross_package_fixture_compiles_and_runs() {
    let fixture = fixture_root("container_cross_package").join("app");

    let compile_output = compile_app_keep_build_dir_expect_success(&fixture);
    assert_artifact_paths_exist(&compile_output);

    let run_output = compile_and_run_app(&fixture);
    assert_exit_code(&run_output, 0);
}

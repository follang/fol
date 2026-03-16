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

fn run_fol(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fol"))
        .args(args)
        .output()
        .expect("should run fol CLI")
}

fn compile_app(entry: &Path) -> std::process::Output {
    run_fol(&[entry.to_str().expect("fixture path should be valid utf-8")])
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

fn compile_and_run_app(entry: &Path) -> std::process::Output {
    let compile_output = compile_app_expect_success(entry);
    let binary = built_binary_path(&compile_output);
    Command::new(&binary)
        .output()
        .expect("should run compiled app binary")
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

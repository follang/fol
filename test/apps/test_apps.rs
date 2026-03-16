use std::path::Path;

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

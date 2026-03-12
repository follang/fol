use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_parser_package_units_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

fn write_folder_fixture(root: &std::path::Path, files: &[(&str, &str)]) {
    fs::create_dir_all(root).expect("Should create temporary parser package fixture");
    for (name, source) in files {
        let path = root.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Should create parser package fixture folder");
        }
        fs::write(path, source).expect("Should write parser package fixture file");
    }
}

#[test]
fn test_parse_package_groups_top_level_items_by_source_unit() {
    let temp_root = unique_temp_root("grouping");
    write_folder_fixture(
        &temp_root,
        &[("00_alpha.fol", "var alpha = 1\n"), ("10_beta.fol", "var beta = 2\n")],
    );

    let parsed = parse_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary parser package fixture path should be UTF-8"),
    );

    fs::remove_dir_all(&temp_root).ok();

    let expected_package = temp_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("Temp parser package fixture should have a UTF-8 folder name")
        .to_string();

    assert_eq!(parsed.package, expected_package);
    assert_eq!(
        parsed.source_units.len(),
        2,
        "Folder parsing should retain one parsed source unit per physical file"
    );
    assert!(matches!(
        source_unit_nodes(&parsed.source_units[0]).as_slice(),
        [AstNode::VarDecl { name, .. }] if name == "alpha"
    ));
    assert!(matches!(
        source_unit_nodes(&parsed.source_units[1]).as_slice(),
        [AstNode::VarDecl { name, .. }] if name == "beta"
    ));
}

#[test]
fn test_parse_package_retains_namespace_per_source_unit() {
    let temp_root = unique_temp_root("namespaces");
    write_folder_fixture(
        &temp_root,
        &[
            ("00_root.fol", "var root_value = 1\n"),
            ("printing/logg/entry.fol", "var warn_value = 2\n"),
        ],
    );

    let parsed = parse_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary parser namespace fixture path should be UTF-8"),
    );

    fs::remove_dir_all(&temp_root).ok();

    let expected_package = temp_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("Temp parser namespace fixture should have a UTF-8 folder name")
        .to_string();

    assert_eq!(parsed.source_units[0].namespace, expected_package);
    assert_eq!(
        parsed.source_units[1].namespace,
        format!("{expected_package}::printing::logg"),
        "Nested folder files should retain the namespace path computed by fol-stream"
    );
    assert!(matches!(
        source_unit_nodes(&parsed.source_units[1]).as_slice(),
        [AstNode::VarDecl { name, .. }] if name == "warn_value"
    ));
}

#[test]
fn test_parse_package_source_unit_order_matches_stream_traversal_order() {
    let temp_root = unique_temp_root("ordering");
    write_folder_fixture(
        &temp_root,
        &[
            ("00_root.fol", "var root_first = 1\n"),
            ("a_nested/00_alpha.fol", "var nested_alpha = 2\n"),
            ("a_nested/10_beta.fol", "var nested_beta = 3\n"),
            ("b_more/00_gamma.fol", "var nested_gamma = 4\n"),
            ("m_root.fol", "var root_last = 5\n"),
        ],
    );

    let parsed = parse_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary parser ordering fixture path should be UTF-8"),
    );

    let canonical_root = std::fs::canonicalize(&temp_root)
        .expect("Ordering fixture root should canonicalize");

    fs::remove_dir_all(&temp_root).ok();

    let relative_paths: Vec<String> = parsed
        .source_units
        .iter()
        .map(|unit| {
            std::path::Path::new(&unit.path)
                .strip_prefix(&canonical_root)
                .expect("Parsed source unit should stay under the fixture root")
                .to_string_lossy()
                .to_string()
        })
        .collect();

    assert_eq!(
        relative_paths,
        vec![
            "00_root.fol",
            "a_nested/00_alpha.fol",
            "a_nested/10_beta.fol",
            "b_more/00_gamma.fol",
            "m_root.fol",
        ],
        "Parsed source units should preserve the exact deterministic traversal order from fol-stream"
    );
    assert!(matches!(
        source_unit_nodes(&parsed.source_units[0]).as_slice(),
        [AstNode::VarDecl { name, .. }] if name == "root_first"
    ));
    assert!(matches!(
        source_unit_nodes(&parsed.source_units[4]).as_slice(),
        [AstNode::VarDecl { name, .. }] if name == "root_last"
    ));
}

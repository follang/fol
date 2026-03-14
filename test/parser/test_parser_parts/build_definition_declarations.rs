use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_build_definition_decl_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

#[test]
fn test_package_parser_accepts_pkg_definition_declarations() {
    let temp_root = unique_temp_root("pkg_definition");
    fs::create_dir_all(&temp_root).expect("Should create temporary parser fixture root");
    let file_path = temp_root.join("build.fol");
    fs::write(&file_path, "def core: pkg = \"core\";\n")
        .expect("Should write the pkg definition fixture");

    let parsed = parse_package_from_file(
        file_path
            .to_str()
            .expect("Temporary parser fixture path should be valid UTF-8"),
    );
    let source_unit = parsed
        .source_units
        .first()
        .expect("Build definition fixture should produce one source unit");

    assert!(
        source_unit.items.iter().any(|item| {
            matches!(
                &item.node,
                AstNode::DefDecl {
                    name,
                    def_type: FolType::Package { name: kind },
                    body,
                    ..
                } if name == "core"
                    && kind.is_empty()
                    && matches!(
                        body.as_slice(),
                        [AstNode::Literal(Literal::String(value))] if value == "core"
                    )
            )
        }),
        "Package parser should accept pkg definition declarations for build surfaces",
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary parser fixture root should be removable after the test");
}

#[test]
fn test_package_parser_accepts_loc_definition_declarations() {
    let temp_root = unique_temp_root("loc_definition");
    fs::create_dir_all(&temp_root).expect("Should create temporary parser fixture root");
    let file_path = temp_root.join("build.fol");
    fs::write(&file_path, "def root: loc = \"src/fmt\";\n")
        .expect("Should write the loc definition fixture");

    let parsed = parse_package_from_file(
        file_path
            .to_str()
            .expect("Temporary parser fixture path should be valid UTF-8"),
    );
    let source_unit = parsed
        .source_units
        .first()
        .expect("Build definition fixture should produce one source unit");

    assert!(
        source_unit.items.iter().any(|item| {
            matches!(
                &item.node,
                AstNode::DefDecl {
                    name,
                    def_type: FolType::Location { name: kind },
                    body,
                    ..
                } if name == "root"
                    && kind.is_empty()
                    && matches!(
                        body.as_slice(),
                        [AstNode::Literal(Literal::String(value))] if value == "src/fmt"
                    )
            )
        }),
        "Package parser should accept loc definition declarations for build surfaces",
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary parser fixture root should be removable after the test");
}

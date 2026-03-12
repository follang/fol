use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fol_parser_source_origins_{}_{}_{}",
        label,
        std::process::id(),
        stamp
    ))
}

#[test]
fn test_parse_package_retains_successful_top_level_origins() {
    let temp_root = unique_temp_root("top_level");
    fs::create_dir_all(&temp_root).expect("Should create temporary source-origin fixture dir");
    let fixture = temp_root.join("origins.fol");
    fs::write(&fixture, "`doc`\nvar alpha = 1\nfun beta(): int = { return alpha }\n")
        .expect("Should write temporary source-origin fixture");

    let parsed = parse_package_from_file(
        fixture
            .to_str()
            .expect("Temporary source-origin fixture path should be UTF-8"),
    );

    let expected_path = std::fs::canonicalize(&fixture)
        .expect("Source-origin fixture path should canonicalize")
        .to_string_lossy()
        .to_string();

    fs::remove_dir_all(&temp_root).ok();

    let source_unit = parsed
        .source_units
        .first()
        .expect("Single-file parse package should expose one source unit");
    assert_eq!(
        source_unit.items.len(),
        3,
        "Comment, binding, and routine should all retain successful origins"
    );

    let comment_origin = parsed_top_level_origin(&parsed, &source_unit.items[0]);
    let var_origin = parsed_top_level_origin(&parsed, &source_unit.items[1]);
    let fun_origin = parsed_top_level_origin(&parsed, &source_unit.items[2]);

    assert_eq!(comment_origin.file.as_deref(), Some(expected_path.as_str()));
    assert_eq!(comment_origin.line, 1);
    assert_eq!(comment_origin.column, 1);

    assert_eq!(var_origin.file.as_deref(), Some(expected_path.as_str()));
    assert_eq!(var_origin.line, 2);
    assert_eq!(var_origin.column, 1);

    assert_eq!(fun_origin.file.as_deref(), Some(expected_path.as_str()));
    assert_eq!(fun_origin.line, 3);
    assert_eq!(fun_origin.column, 1);
}

#[test]
fn test_parse_package_retains_item_origins_across_multiple_files() {
    let temp_root = unique_temp_root("multi_file");
    fs::create_dir_all(&temp_root).expect("Should create temporary multi-file origin fixture dir");
    let alpha = temp_root.join("00_alpha.fol");
    let beta = temp_root.join("10_beta.fol");
    fs::write(&alpha, "var alpha = 1\n").expect("Should write first multi-file origin fixture");
    fs::write(&beta, "var beta = 2\n").expect("Should write second multi-file origin fixture");

    let parsed = parse_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary multi-file origin fixture path should be UTF-8"),
    );

    fs::remove_dir_all(&temp_root).ok();

    let first_origin = parsed_top_level_origin(&parsed, &parsed.source_units[0].items[0]);
    let second_origin = parsed_top_level_origin(&parsed, &parsed.source_units[1].items[0]);

    assert_ne!(
        first_origin.file,
        second_origin.file,
        "Parsed top-level origins should keep the physical file identity of each source unit"
    );
    assert_eq!(first_origin.line, 1);
    assert_eq!(second_origin.line, 1);
}

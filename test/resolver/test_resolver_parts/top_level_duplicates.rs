use super::{try_resolve_package_from_folder, unique_temp_root};
use std::fs;

#[test]
fn test_resolver_rejects_raw_duplicate_top_level_symbols() {
    let temp_root = unique_temp_root("duplicate_symbols");
    fs::create_dir_all(&temp_root).expect("Should create duplicate-symbol fixture dir");
    fs::write(temp_root.join("00_first.fol"), "var value: int = 1;\n")
        .expect("Should write first duplicate fixture");
    fs::write(temp_root.join("01_second.fol"), "var value: int = 2;\n")
        .expect("Should write second duplicate fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Resolver folder fixture path should be utf-8"),
    )
    .expect_err("Duplicate top-level symbols should fail resolver collection");

    assert!(
        errors
            .iter()
            .any(|error| error.to_string().contains("duplicate symbol 'value'")),
        "Expected duplicate top-level symbol wording for exact duplicate names"
    );
    assert!(
        errors.iter().any(|error| error
            .origin()
            .and_then(|origin| origin.file.as_ref())
            .is_some()),
        "Duplicate symbol errors should retain the conflicting source file"
    );

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn test_resolver_rejects_canonical_duplicate_top_level_symbols_across_files() {
    let temp_root = unique_temp_root("canonical_duplicates");
    fs::create_dir_all(&temp_root).expect("Should create canonical-duplicate fixture dir");
    fs::write(temp_root.join("00_first.fol"), "typ Value_Name: int;\n")
        .expect("Should write first canonical duplicate fixture");
    fs::write(temp_root.join("01_second.fol"), "typ valueName: str;\n")
        .expect("Should write second canonical duplicate fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Resolver folder fixture path should be utf-8"),
    )
    .expect_err("Canonical duplicate top-level symbols should fail resolver collection");

    assert!(
        errors
            .iter()
            .any(|error| error.to_string().contains("duplicate symbol 'valueName'")),
        "Expected duplicate detection to treat ASCII case and underscores canonically"
    );

    fs::remove_dir_all(&temp_root).ok();
}

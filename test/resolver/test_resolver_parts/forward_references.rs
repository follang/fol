use super::{resolve_package_from_folder, unique_temp_root};
use fol_resolver::{ReferenceKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_allows_forward_top_level_value_references_in_package_scope() {
    let temp_root = unique_temp_root("forward_top_level_value");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(temp_root.join("00_value_use.fol"), "var first: int = later;\n")
        .expect("Should write the forward value-use fixture");
    fs::write(temp_root.join("01_value_def.fol"), "var later: int = 42;\n")
        .expect("Should write the forward value-definition fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let source_unit_scope = resolved
        .source_units
        .iter()
        .next()
        .expect("Resolver should keep the source unit")
        .scope_id;
    let later_symbol = resolved
        .symbols_in_scope(resolved.program_scope)
        .into_iter()
        .find(|symbol| symbol.name == "later" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Program scope should keep the forward-declared value symbol");

    assert!(
        resolved
            .references_in_scope(source_unit_scope)
            .into_iter()
            .any(|reference| {
                reference.kind == ReferenceKind::Identifier
                    && reference.name == "later"
                    && reference.resolved == Some(later_symbol.id)
            }),
        "Top-level value initializers should resolve forward references after whole-package collection"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_allows_forward_top_level_type_references_in_package_scope() {
    let temp_root = unique_temp_root("forward_top_level_type");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    fs::write(temp_root.join("00_type_use.fol"), "ali Result: Later;\n")
        .expect("Should write the forward type-use fixture");
    fs::write(temp_root.join("01_type_def.fol"), "typ Later: int;\n")
        .expect("Should write the forward type-definition fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    );
    let source_unit_scope = resolved
        .source_units
        .iter()
        .next()
        .expect("Resolver should keep the source unit")
        .scope_id;
    let later_symbol = resolved
        .symbols_in_scope(resolved.program_scope)
        .into_iter()
        .find(|symbol| symbol.name == "Later" && symbol.kind == SymbolKind::Type)
        .expect("Program scope should keep the forward-declared type symbol");

    assert!(
        resolved
            .references_in_scope(source_unit_scope)
            .into_iter()
            .any(|reference| {
                reference.kind == ReferenceKind::TypeName
                    && reference.name == "Later"
                    && reference.resolved == Some(later_symbol.id)
            }),
        "Top-level type aliases should resolve forward type references after whole-package collection"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

use super::{resolve_package_from_file, try_resolve_package_from_folder, unique_temp_root};
use fol_parser::ast::AstNode;
use fol_resolver::{ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_builds_routine_scope_for_named_routines_and_nested_named_routines() {
    let resolved = resolve_package_from_file("test/parser/simple_fun_named_closure_capture.fol");
    let package = resolved.syntax();
    let root_item = package
        .source_units
        .first()
        .and_then(|unit| unit.items.first())
        .expect("Named closure capture fixture should have a root routine");
    let root_syntax_id = root_item
        .node
        .syntax_id()
        .expect("Root named routine should keep a syntax id");
    let root_scope_id = resolved
        .scope_for_syntax(root_syntax_id)
        .expect("Resolver should create a scope for the root routine");
    let root_scope = resolved
        .scope(root_scope_id)
        .expect("Root routine scope should exist");
    let source_unit_scope = resolved
        .source_units
        .iter()
        .next()
        .expect("Resolver should keep the source unit")
        .scope_id;

    assert!(matches!(root_scope.kind, ScopeKind::Routine));
    assert_eq!(root_scope.parent, Some(source_unit_scope));
    assert!(
        resolved
            .symbols_in_scope(root_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "n" && symbol.kind == SymbolKind::Parameter),
        "Routine scopes should bind routine parameters"
    );
    assert!(
        resolved
            .symbols_in_scope(root_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "added" && symbol.kind == SymbolKind::Routine),
        "Nested named routines should bind into the enclosing routine scope"
    );

    let nested_syntax_id = match &root_item.node {
        AstNode::FunDecl { body, .. } => body
            .iter()
            .find_map(|node| match node {
                AstNode::FunDecl { syntax_id, .. } => *syntax_id,
                _ => None,
            })
            .expect("Fixture should contain a nested named routine"),
        node => panic!("Expected root named routine, got {:?}", node),
    };
    let nested_scope_id = resolved
        .scope_for_syntax(nested_syntax_id)
        .expect("Nested named routine should have a routine scope");
    let nested_scope = resolved
        .scope(nested_scope_id)
        .expect("Nested routine scope should exist");

    assert!(matches!(nested_scope.kind, ScopeKind::Routine));
    assert_eq!(nested_scope.parent, Some(root_scope_id));
    assert!(
        resolved
            .symbols_in_scope(nested_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "n" && symbol.kind == SymbolKind::Capture),
        "Nested routine scopes should bind declared captures"
    );
    assert!(
        resolved
            .symbols_in_scope(nested_scope_id)
            .into_iter()
            .any(|symbol| symbol.name == "x" && symbol.kind == SymbolKind::Parameter),
        "Nested routine scopes should bind their own parameters"
    );
}

#[test]
fn test_resolver_rejects_duplicate_capture_and_parameter_bindings_in_routine_scope() {
    let temp_root = unique_temp_root("routine_scope_duplicates");
    fs::create_dir_all(&temp_root).expect("Should create a temporary resolver fixture directory");
    let main_file = temp_root.join("main.fol");
    fs::write(
        &main_file,
        "fun[] add(x: int)[x]: int = {\n    return x;\n};\n",
    )
    .expect("Should write the duplicate routine binding fixture");

    let errors = try_resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
    )
    .expect_err("Resolver should reject duplicate local routine bindings");

    assert!(
        errors
            .iter()
            .any(|error| error.kind() == ResolverErrorKind::DuplicateSymbol),
        "Resolver should report duplicate local symbol errors for capture/parameter collisions"
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

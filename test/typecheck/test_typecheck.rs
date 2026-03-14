use fol_diagnostics::{DiagnosticCode, DiagnosticLocation, ToDiagnostic};
use fol_parser::ast::{AstParser, SyntaxOrigin};
use fol_resolver::resolve_package;
use fol_resolver::SymbolId;
use fol_stream::FileStream;
use fol_typecheck::{
    BuiltinType, BuiltinTypeIds, CheckedType, DeclaredTypeKind, RoutineType, TypeTable,
    TypecheckError, TypecheckErrorKind, Typechecker,
};
use std::collections::BTreeMap;

fn resolve_fixture(path: &str) -> fol_resolver::ResolvedProgram {
    let fixture_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
    let mut stream = FileStream::from_file(&fixture_path).expect("Should open typecheck fixture");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Typecheck fixture should parse as a package");

    resolve_package(syntax).expect("Typecheck fixture should resolve cleanly")
}

#[test]
fn typechecker_foundation_smoke_constructs_public_api() {
    let _ = Typechecker::new();
}

#[test]
fn typecheck_errors_keep_exact_diagnostic_locations() {
    let error = TypecheckError::with_origin(
        TypecheckErrorKind::InvalidInput,
        "declared type is not valid in this position",
        SyntaxOrigin {
            file: Some("pkg/main.fol".to_string()),
            line: 5,
            column: 9,
            length: 3,
        },
    );

    assert_eq!(
        error.diagnostic_location(),
        Some(DiagnosticLocation {
            file: Some("pkg/main.fol".to_string()),
            line: 5,
            column: 9,
            length: Some(3),
        })
    );
}

#[test]
fn typecheck_errors_lower_to_stable_structured_diagnostics() {
    let diagnostic = TypecheckError::with_origin(
        TypecheckErrorKind::Unsupported,
        "blueprints are not part of the V1 typecheck milestone",
        SyntaxOrigin {
            file: Some("pkg/main.fol".to_string()),
            line: 2,
            column: 1,
            length: 3,
        },
    )
    .with_related_origin(
        SyntaxOrigin {
            file: Some("pkg/std.fol".to_string()),
            line: 1,
            column: 1,
            length: 3,
        },
        "related declaration site",
    )
    .to_diagnostic();

    assert_eq!(diagnostic.code, DiagnosticCode::new("T1002"));
    assert_eq!(
        diagnostic.primary_location(),
        Some(&DiagnosticLocation {
            file: Some("pkg/main.fol".to_string()),
            line: 2,
            column: 1,
            length: Some(3),
        })
    );
    assert_eq!(diagnostic.labels.len(), 2);
}

#[test]
fn builtin_type_tables_install_v1_scalar_types_canonically() {
    let mut table = TypeTable::new();
    let builtins = BuiltinTypeIds::install(&mut table);

    assert_eq!(table.len(), 6);
    assert_eq!(table.get(builtins.int), Some(&CheckedType::Builtin(BuiltinType::Int)));
    assert_eq!(
        table.get(builtins.str_),
        Some(&CheckedType::Builtin(BuiltinType::Str))
    );
}

#[test]
fn builtin_type_installation_reuses_existing_slots() {
    let mut table = TypeTable::new();
    let first = BuiltinTypeIds::install(&mut table);
    let second = BuiltinTypeIds::install(&mut table);

    assert_eq!(first, second);
    assert_eq!(table.len(), 6);
}

#[test]
fn typechecker_wraps_resolved_programs_in_a_typed_shell() {
    let resolved = resolve_fixture("test/parser/simple_var.fol");
    let top_level_node = resolved
        .source_units
        .get(fol_resolver::SourceUnitId(0))
        .expect("resolved source unit should exist")
        .top_level_nodes[0];
    let typed = Typechecker::new()
        .check_resolved_program(resolved)
        .expect("Typed shell should accept resolved programs");

    assert_eq!(typed.package_name(), "parser");
    assert_eq!(typed.source_units().len(), 1);
    assert_eq!(typed.type_table().len(), 6);
    assert_eq!(
        typed.type_table().get(typed.builtin_types().bool_),
        Some(&CheckedType::Builtin(BuiltinType::Bool))
    );
    assert_eq!(typed.resolved().source_units.len(), 1);
    assert!(typed.typed_node(top_level_node).is_some());
    assert!(typed.typed_symbol(SymbolId(0)).is_some());
}

#[test]
fn semantic_type_table_covers_declared_and_structural_shapes() {
    let mut table = TypeTable::new();
    let int_id = table.intern_builtin(BuiltinType::Int);
    let alias_id = table.intern(CheckedType::Declared {
        symbol: SymbolId(9),
        name: "Meters".to_string(),
        kind: DeclaredTypeKind::Alias,
    });

    let mut fields = BTreeMap::new();
    fields.insert("value".to_string(), alias_id);
    let record = table.intern(CheckedType::Record { fields });
    let routine = table.intern(CheckedType::Routine(RoutineType {
        params: vec![alias_id],
        return_type: int_id,
        error_type: None,
    }));

    assert_ne!(record, routine);
    assert_eq!(
        table.get(alias_id),
        Some(&CheckedType::Declared {
            symbol: SymbolId(9),
            name: "Meters".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
}

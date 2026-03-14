use fol_diagnostics::{DiagnosticCode, DiagnosticLocation, ToDiagnostic};
use fol_parser::ast::{AstParser, SyntaxOrigin};
use fol_resolver::resolve_package;
use fol_resolver::{SourceUnitId, SymbolId, SymbolKind};
use fol_stream::FileStream;
use fol_typecheck::{
    BuiltinType, BuiltinTypeIds, CheckedType, DeclaredTypeKind, RoutineType, TypeTable,
    TypecheckError, TypecheckErrorKind, Typechecker,
};
use std::collections::BTreeMap;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("fol_typecheck_{prefix}_{nonce}"))
}

fn write_fixture_files(root: &Path, files: &[(&str, &str)]) {
    for (relative_path, contents) in files {
        let path = root.join(relative_path);
        if let Some(parent) = path.parent() {
            create_dir_all(parent).expect("Fixture parent directories should be creatable");
        }
        write(&path, contents).expect("Fixture file should be writable");
    }
}

fn typecheck_fixture_folder(files: &[(&str, &str)]) -> fol_typecheck::TypedProgram {
    let root = unique_temp_dir("package");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(&root, files);

    let mut stream = FileStream::from_folder(root.to_str().expect("fixture path should be utf8"))
        .expect("Fixture folder should stream");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Fixture folder should parse as a package");
    let resolved = resolve_package(syntax).expect("Fixture folder should resolve cleanly");

    Typechecker::new()
        .check_resolved_program(resolved)
        .expect("Fixture folder should typecheck declaration signatures")
}

fn find_typed_symbol<'a>(
    typed: &'a fol_typecheck::TypedProgram,
    name: &str,
    kind: SymbolKind,
) -> (SymbolId, &'a fol_typecheck::TypedSymbol) {
    let symbol_id = typed
        .resolved()
        .symbols
        .iter_with_ids()
        .find(|(_, symbol)| symbol.name == name && symbol.kind == kind)
        .map(|(symbol_id, _)| symbol_id)
        .expect("typed fixture symbol should exist");
    let symbol = typed
        .typed_symbol(symbol_id)
        .expect("typed symbol facts should exist for resolved symbol");

    (symbol_id, symbol)
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
        return_type: Some(int_id),
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

#[test]
fn declaration_signature_lowering_records_top_level_type_facts() {
    let typed = typecheck_fixture_folder(&[
        (
            "types.fol",
            "ali Distance: int\n\
             typ Person: rec = {\n\
                 name: str\n\
             }\n",
        ),
        (
            "main.fol",
            "var total: Distance = 1\n\
             fun[] size(value: Distance): Person = {\n\
                 return total\n\
             }\n",
        ),
    ]);

    let (distance_id, distance) = find_typed_symbol(&typed, "Distance", SymbolKind::Alias);
    let (person_id, person) = find_typed_symbol(&typed, "Person", SymbolKind::Type);
    let (_size_id, size) = find_typed_symbol(&typed, "size", SymbolKind::Routine);

    assert_eq!(
        typed.type_table().get(distance.declared_type.expect("alias should lower")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
    assert_eq!(
        typed.type_table().get(person.declared_type.expect("record should lower")),
        Some(&CheckedType::Record {
            fields: BTreeMap::from([("name".to_string(), typed.builtin_types().str_)])
        })
    );
    let routine_type_id = size.declared_type.expect("routine should lower");
    let routine_type = typed
        .type_table()
        .get(routine_type_id)
        .expect("lowered routine type should exist");
    let CheckedType::Routine(routine) = routine_type else {
        panic!("lowered routine signature should be represented as a routine type");
    };
    assert_eq!(routine.error_type, None);
    assert_eq!(routine.params.len(), 1);
    assert_eq!(
        typed.type_table().get(routine.params[0]),
        Some(&CheckedType::Declared {
            symbol: distance_id,
            name: "Distance".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    assert_eq!(
        typed.type_table().get(routine.return_type.expect("routine return type should lower")),
        Some(&CheckedType::Declared {
            symbol: person_id,
            name: "Person".to_string(),
            kind: DeclaredTypeKind::Type,
        })
    );
    assert_eq!(typed.resolved().source_units.get(SourceUnitId(0)).map(|unit| unit.package.as_str()), Some(typed.package_name()));
}

#[test]
fn declaration_signature_lowering_keeps_builtin_str_types_builtin() {
    let typed = typecheck_fixture_folder(&[("main.fol", "var label: str = \"ok\"\n")]);
    let (_label_id, label) = find_typed_symbol(&typed, "label", SymbolKind::ValueBinding);

    assert_eq!(
        typed.type_table().get(label.declared_type.expect("binding should lower")),
        Some(&CheckedType::Builtin(BuiltinType::Str))
    );
}

#[test]
fn declaration_signature_lowering_keeps_named_types_as_declared_symbols() {
    let typed = typecheck_fixture_folder(&[
        ("types.fol", "typ Point: rec = {\n}\n"),
        ("main.fol", "var current: Point = nil\n"),
    ]);

    let (point_id, _point) = find_typed_symbol(&typed, "Point", SymbolKind::Type);
    let (_current_id, current) = find_typed_symbol(&typed, "current", SymbolKind::ValueBinding);

    assert_eq!(
        typed
            .type_table()
            .get(current.declared_type.expect("binding should lower")),
        Some(&CheckedType::Declared {
            symbol: point_id,
            name: "Point".to_string(),
            kind: DeclaredTypeKind::Type,
        })
    );
}

#[test]
fn declaration_signature_lowering_keeps_alias_references_as_alias_symbols() {
    let typed = typecheck_fixture_folder(&[
        ("types.fol", "ali Count: int\n"),
        ("main.fol", "var total: Count = 1\n"),
    ]);

    let (count_id, _count) = find_typed_symbol(&typed, "Count", SymbolKind::Alias);
    let (_total_id, total) = find_typed_symbol(&typed, "total", SymbolKind::ValueBinding);

    assert_eq!(
        typed.type_table().get(total.declared_type.expect("binding should lower")),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
}

#[test]
fn declaration_signature_lowering_resolves_qualified_named_types() {
    let typed = typecheck_fixture_folder(&[
        ("util/types.fol", "ali Count: int\n"),
        ("main.fol", "var total: util::Count = 1\n"),
    ]);

    let (count_id, _count) = find_typed_symbol(&typed, "Count", SymbolKind::Alias);
    let (_total_id, total) = find_typed_symbol(&typed, "total", SymbolKind::ValueBinding);

    assert_eq!(
        typed.type_table().get(total.declared_type.expect("binding should lower")),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
}

#[test]
fn declaration_signature_lowering_checks_local_bindings() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] demo(): int = {\n\
             var local: int = 1;\n\
             return local;\n\
         }\n",
    )]);

    let (_local_id, local) = find_typed_symbol(&typed, "local", SymbolKind::ValueBinding);

    assert_eq!(
        typed.type_table().get(local.declared_type.expect("local binding should lower")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn declaration_signature_lowering_checks_nested_routine_signatures() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] demo(seed: int): int = {\n\
             fun[] helper(item: str): int = {\n\
                 return seed;\n\
             };\n\
             return seed;\n\
         }\n",
    )]);

    let (_helper_id, helper) = find_typed_symbol(&typed, "helper", SymbolKind::Routine);
    let helper_type = typed
        .type_table()
        .get(helper.declared_type.expect("nested routine should lower"))
        .expect("nested routine type should exist");
    let CheckedType::Routine(helper_type) = helper_type else {
        panic!("nested routine should lower to a routine type");
    };

    assert_eq!(helper_type.error_type, None);
    assert_eq!(
        typed.type_table().get(helper_type.params[0]),
        Some(&CheckedType::Builtin(BuiltinType::Str))
    );
    assert_eq!(
        typed
            .type_table()
            .get(helper_type.return_type.expect("nested routine return should lower")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

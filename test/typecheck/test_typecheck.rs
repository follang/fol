use fol_diagnostics::{DiagnosticCode, DiagnosticLocation, ToDiagnostic};
use fol_parser::ast::{AstParser, SyntaxOrigin};
use fol_resolver::{resolve_package, resolve_package_workspace_with_config};
use fol_resolver::{ReferenceKind, ResolverConfig, SourceUnitId, SymbolId, SymbolKind};
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

fn typecheck_fixture_folder_errors(files: &[(&str, &str)]) -> Vec<TypecheckError> {
    let root = unique_temp_dir("package_errors");
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
        .expect_err("Fixture folder should fail typechecking")
}

fn typecheck_fixture_entry_with_config(
    root: &Path,
    entry: &str,
    config: ResolverConfig,
) -> Result<fol_typecheck::TypedProgram, Vec<TypecheckError>> {
    let entry_root = root.join(entry);
    let mut stream =
        FileStream::from_folder(entry_root.to_str().expect("fixture path should be utf8"))
            .expect("Fixture folder should stream");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Fixture folder should parse as a package");
    let resolved = fol_resolver::resolve_package_with_config(syntax, config)
        .expect("Fixture folder should resolve cleanly");

    Typechecker::new().check_resolved_program(resolved)
}

fn typecheck_fixture_workspace_with_config(
    root: &Path,
    entry: &str,
    config: ResolverConfig,
) -> Result<fol_typecheck::TypedWorkspace, Vec<TypecheckError>> {
    let entry_root = root.join(entry);
    let mut stream =
        FileStream::from_folder(entry_root.to_str().expect("fixture path should be utf8"))
            .expect("Fixture folder should stream");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Fixture folder should parse as a package");
    let resolved = resolve_package_workspace_with_config(syntax, config)
        .expect("Fixture folder should resolve cleanly as a workspace");

    Typechecker::new().check_resolved_workspace(resolved)
}

fn typecheck_fixture_workspace_entry_with_config(
    root: &Path,
    entry: &str,
    config: ResolverConfig,
) -> Result<fol_typecheck::TypedProgram, Vec<TypecheckError>> {
    typecheck_fixture_workspace_with_config(root, entry, config)
        .map(|typed| typed.entry_program().clone())
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

fn find_typed_package<'a>(
    typed: &'a fol_typecheck::TypedWorkspace,
    display_name: &str,
) -> &'a fol_typecheck::TypedPackage {
    typed
        .packages()
        .find(|package| package.identity.display_name == display_name)
        .expect("typed workspace package should exist")
}

fn find_typed_reference<'a>(
    typed: &'a fol_typecheck::TypedProgram,
    name: &str,
    kind: ReferenceKind,
) -> &'a fol_typecheck::TypedReference {
    let reference_id = typed
        .resolved()
        .references
        .iter_with_ids()
        .find(|(_, reference)| reference.name == name && reference.kind == kind)
        .map(|(reference_id, _)| reference_id)
        .expect("typed fixture reference should exist");
    typed
        .typed_reference(reference_id)
        .expect("typed reference facts should exist")
}

fn find_typed_references<'a>(
    typed: &'a fol_typecheck::TypedProgram,
    name: &str,
    kind: ReferenceKind,
) -> Vec<&'a fol_typecheck::TypedReference> {
    typed
        .resolved()
        .references
        .iter_with_ids()
        .filter(|(_, reference)| reference.name == name && reference.kind == kind)
        .map(|(reference_id, _)| {
            typed
                .typed_reference(reference_id)
                .expect("typed reference facts should exist")
        })
        .collect()
}

fn assert_imported_declared_count_binding_and_routine(
    typed: &fol_typecheck::TypedProgram,
    count_symbol: SymbolKind,
) {
    let (count_id, _count) = find_typed_symbol(typed, "Count", count_symbol);
    let (_answer_id, answer) = find_typed_symbol(typed, "answer", SymbolKind::ValueBinding);
    let (_bump_id, bump) = find_typed_symbol(typed, "bump", SymbolKind::Routine);

    assert_eq!(
        typed.type_table().get(
            answer
                .declared_type
                .expect("imported values should keep declared semantic types"),
        ),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: match count_symbol {
                SymbolKind::Alias => DeclaredTypeKind::Alias,
                _ => DeclaredTypeKind::Type,
            },
        })
    );

    let signature = match typed.type_table().get(
        bump.declared_type
            .expect("imported routines should keep translated signatures"),
    ) {
        Some(CheckedType::Routine(signature)) => signature,
        other => panic!("expected imported routine signature, got {other:?}"),
    };
    assert_eq!(signature.params.len(), 1);
    assert_eq!(signature.error_type, None);
    assert_eq!(
        typed.type_table().get(signature.params[0]),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: match count_symbol {
                SymbolKind::Alias => DeclaredTypeKind::Alias,
                _ => DeclaredTypeKind::Type,
            },
        })
    );
    assert_eq!(
        signature
            .return_type
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: match count_symbol {
                SymbolKind::Alias => DeclaredTypeKind::Alias,
                _ => DeclaredTypeKind::Type,
            },
        })
    );
}

fn find_named_routine_syntax_id(
    typed: &fol_typecheck::TypedProgram,
    name: &str,
) -> fol_parser::ast::SyntaxNodeId {
    typed
        .resolved()
        .syntax()
        .source_units
        .iter()
        .flat_map(|unit| unit.items.iter())
        .find_map(|item| match &item.node {
            fol_parser::ast::AstNode::FunDecl {
                name: routine_name,
                syntax_id: Some(syntax_id),
                ..
            }
            | fol_parser::ast::AstNode::ProDecl {
                name: routine_name,
                syntax_id: Some(syntax_id),
                ..
            }
            | fol_parser::ast::AstNode::LogDecl {
                name: routine_name,
                syntax_id: Some(syntax_id),
                ..
            } if routine_name == name => Some(*syntax_id),
            _ => None,
        })
        .expect("named routine syntax id should exist")
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
             var holder: Person\n\
             fun[] size(value: Distance): Person = {\n\
                 return holder\n\
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
        ("main.fol", "var current: Point\n"),
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
fn expression_typing_resolves_plain_identifier_references_to_declared_types() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "var total: int = 1\n\
         fun[] read(): int = {\n\
             return total;\n\
         }\n",
    )]);

    let reference = find_typed_reference(&typed, "total", ReferenceKind::Identifier);

    assert_eq!(
        typed
            .type_table()
            .get(reference.resolved_type.expect("identifier should receive a type")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_resolves_qualified_identifier_references_to_declared_types() {
    let typed = typecheck_fixture_folder(&[
        ("util/value.fol", "var[exp] total: int = 1\n"),
        (
            "main.fol",
            "fun[] read(): int = {\n\
                 return util::total;\n\
             }\n",
        ),
    ]);

    let reference = find_typed_reference(&typed, "util::total", ReferenceKind::QualifiedIdentifier);

    assert_eq!(
        typed
            .type_table()
            .get(reference.resolved_type.expect("qualified identifier should receive a type")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_infers_local_binding_types_from_initializers() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] demo(): int = {\n\
             let current = 1;\n\
             return current;\n\
         }\n",
    )]);

    let (_current_id, current) = find_typed_symbol(&typed, "current", SymbolKind::ValueBinding);

    assert_eq!(
        typed
            .type_table()
            .get(current.declared_type.expect("initializer should infer local type")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_keeps_final_routine_body_expression_types() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "var total: int = 1\n\
         fun[] demo(): int = {\n\
             total\n\
         }\n",
    )]);
    let syntax_id = find_named_routine_syntax_id(&typed, "demo");

    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_accepts_assignments_with_matching_types() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "var total: int = 1\n\
         fun[] demo(): int = {\n\
             total = 2;\n\
             return total;\n\
         }\n",
    )]);

    let reference = find_typed_reference(&typed, "total", ReferenceKind::Identifier);
    assert_eq!(
        typed
            .type_table()
            .get(reference.resolved_type.expect("identifier should keep its type after assignment")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_rejects_assignments_with_mismatched_value_types() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "var total: int = 1\n\
         fun[] demo(): int = {\n\
             total = \"bad\";\n\
             return total;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("assignment expects")
        }),
        "Expected an incompatible assignment diagnostic, got: {errors:?}"
    );
}

#[test]
fn expression_typing_types_free_calls_against_routine_signatures() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] id(value: int): int = {\n\
             return value;\n\
         }\n\
         fun[] demo(): int = {\n\
             return id(1);\n\
         }\n",
    )]);

    let reference = find_typed_reference(&typed, "id", ReferenceKind::FunctionCall);
    assert_eq!(
        typed
            .type_table()
            .get(reference.resolved_type.expect("free call should receive a result type")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_rejects_free_call_arity_mismatches() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] id(value: int): int = {\n\
             return value;\n\
         }\n\
         fun[] demo(): int = {\n\
             return id();\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error.message().contains("expects 1 args but got 0")
        }),
        "Expected an arity diagnostic for free call mismatch, got: {errors:?}"
    );
}

#[test]
fn expression_typing_types_method_calls_against_explicit_receiver_routines() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ Counter: rec = {\n\
             value: int\n\
         }\n\
         var current: Counter\n\
         fun (Counter)read(): int = {\n\
             return 1;\n\
         }\n\
         fun[] demo(): int = {\n\
             return current.read();\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "demo");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_rejects_method_call_arity_mismatches() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "typ Counter: rec = {\n\
             value: int\n\
         }\n\
         var current: Counter\n\
         fun (Counter)read(value: int): int = {\n\
             return value;\n\
         }\n\
         fun[] demo(): int = {\n\
             return current.read();\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error.message().contains("expects 1 args but got 0")
        }),
        "Expected an arity diagnostic for method call mismatch, got: {errors:?}"
    );
}

#[test]
fn expression_typing_types_field_access_against_named_record_receivers() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ Counter: rec = {\n\
             value: int\n\
         }\n\
         fun[] read(counter: Counter): int = {\n\
             return counter.value;\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "read");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_rejects_field_access_on_non_records() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] bad(value: int): int = {\n\
             return value.total;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("field access '.total' requires a record-like or entry-like receiver")
        }),
        "Expected a non-record field-access diagnostic, got: {errors:?}"
    );
}

#[test]
fn expression_typing_expands_alias_record_shells_for_field_access() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ CounterShape: rec = {\n\
             value: int\n\
         }\n\
         ali Counter: CounterShape\n\
         var current: Counter = { value = 1 }\n\
         fun[] read(): int = {\n\
             return current.value;\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "read");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_types_container_index_accesses() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] head(values: vec[int]): int = {\n\
             return values[0];\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "head");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn expression_typing_types_basic_slice_accesses() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] tail(values: vec[int]): vec[int] = {\n\
             return values[1:];\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "tail");
    let inferred = typed
        .typed_node(syntax_id)
        .and_then(|node| node.inferred_type)
        .and_then(|type_id| typed.type_table().get(type_id));

    assert!(matches!(
        inferred,
        Some(CheckedType::Vector { element_type })
            if typed.type_table().get(*element_type)
                == Some(&CheckedType::Builtin(BuiltinType::Int))
    ));
}

#[test]
fn expression_typing_rejects_non_indexable_receivers() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] bad(value: int): int = {\n\
             return value[0];\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("index access requires an array, vector, sequence, set, or map receiver")
        }),
        "Expected a non-indexable access diagnostic, got: {errors:?}"
    );
}

#[test]
fn routine_return_typing_rejects_explicit_return_mismatches() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] demo(): int = {\n\
             return false;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("return expects")
        }),
        "Expected a return-type mismatch diagnostic, got: {errors:?}"
    );
}

#[test]
fn routine_return_typing_rejects_final_body_expression_mismatches() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "var flag: bol = false\n\
         fun[] demo(): int = {\n\
             flag\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("routine 'demo' body expects")
        }),
        "Expected a routine-body mismatch diagnostic, got: {errors:?}"
    );
}

#[test]
fn routine_return_typing_rejects_missing_return_values_for_typed_routines() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] demo(): int = {\n\
             return;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("return requires a value for routines with a declared return type")
        }),
        "Expected a missing-return-value diagnostic, got: {errors:?}"
    );
}

#[test]
fn routine_error_typing_accepts_matching_report_values() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] demo(): int : str = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "demo");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn routine_error_typing_rejects_report_value_mismatches() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] demo(): int : str = {\n\
             report 1;\n\
             return 1;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("report expects")
        }),
        "Expected a report-type mismatch diagnostic, got: {errors:?}"
    );
}

#[test]
fn routine_error_typing_requires_declared_error_types() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] demo(): int = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("report requires a declared routine error type")
        }),
        "Expected a missing-error-type diagnostic, got: {errors:?}"
    );
}

#[test]
fn routine_error_typing_rejects_missing_report_values() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] demo(): int : str = {\n\
             report;\n\
             return 1;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error.message().contains("report expects exactly 1 value in V1 but got 0")
        }),
        "Expected a missing-report-value diagnostic, got: {errors:?}"
    );
}

#[test]
fn when_result_typing_accepts_matching_branch_values() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "var yes: int = 1\n\
         var no: int = 2\n\
         fun[] demo(flag: bol): int = {\n\
             when(flag) {\n\
                 case(true) { yes }\n\
                 * { no }\n\
             }\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "demo");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn when_result_typing_rejects_branch_type_mismatches() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "var yes: int = 1\n\
         var no: bol = false\n\
         fun[] demo(flag: bol): int = {\n\
             when(flag) {\n\
                 case(true) { yes }\n\
                 * { no }\n\
             }\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("when branch expects")
        }),
        "Expected a when-branch mismatch diagnostic, got: {errors:?}"
    );
}

#[test]
fn loop_typing_infers_iteration_binder_types_and_checks_bool_guards() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] demo(items: seq[int], ready: bol, limit: int): int = {\n\
             loop(item in items when ready) {\n\
                 break;\n\
             }\n\
             return limit;\n\
         }\n",
    )]);

    let (_item_id, item) = find_typed_symbol(&typed, "item", SymbolKind::LoopBinder);

    assert_eq!(
        typed
            .type_table()
            .get(item.declared_type.expect("loop binder should infer an element type")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn loop_typing_rejects_non_boolean_conditions_and_reserved_yields() {
    let condition_errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] bad(limit: int): int = {\n\
             loop(limit) {\n\
                 break;\n\
             }\n\
             return limit;\n\
         }\n",
    )]);

    assert!(
        condition_errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("loop condition expects")
        }),
        "Expected a non-boolean loop condition diagnostic, got: {condition_errors:?}"
    );

    let yield_errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] bad(items: seq[int]): seq[int] = {\n\
             loop(item in items) {\n\
                 yeild item;\n\
             }\n\
             return items;\n\
         }\n",
    )]);

    assert!(
        yield_errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("yeild typing is not part of the V1 typecheck milestone")
        }),
        "Expected an explicit yeild boundary diagnostic, got: {yield_errors:?}"
    );
}

#[test]
fn control_never_typing_allows_panic_and_skips_unreachable_tails() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] demo(): int = {\n\
             panic \"boom\";\n\
             return \"bad\";\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "demo");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn control_never_typing_treats_report_branches_as_early_exits() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] demo(flag: bol): int : str = {\n\
             when(flag) {\n\
                 case(true) { report \"bad\"; }\n\
                 * { 1 }\n\
             }\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "demo");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn container_literal_typing_accepts_array_vector_and_sequence_contexts() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] make_arr(): arr[int, 3] = {\n\
             return {1, 2, 3};\n\
         }\n\
         fun[] make_vec(): vec[int] = {\n\
             return {1, 2, 3};\n\
         }\n\
         fun[] make_seq(): seq[int] = {\n\
             return {1, 2, 3};\n\
         }\n",
    )]);

    for (name, expected_label) in [
        ("make_arr", "Array"),
        ("make_vec", "Vector"),
        ("make_seq", "Sequence"),
    ] {
        let syntax_id = find_named_routine_syntax_id(&typed, name);
        let inferred = typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id));
        assert!(
            matches!(
                inferred,
                Some(CheckedType::Array { element_type, .. })
                    if expected_label == "Array"
                        && typed.type_table().get(*element_type)
                            == Some(&CheckedType::Builtin(BuiltinType::Int))
            ) || matches!(
                inferred,
                Some(CheckedType::Vector { element_type })
                    if expected_label == "Vector"
                        && typed.type_table().get(*element_type)
                            == Some(&CheckedType::Builtin(BuiltinType::Int))
            ) || matches!(
                inferred,
                Some(CheckedType::Sequence { element_type })
                    if expected_label == "Sequence"
                        && typed.type_table().get(*element_type)
                            == Some(&CheckedType::Builtin(BuiltinType::Int))
            ),
            "Expected {name} to keep a {expected_label} container type, got {inferred:?}"
        );
    }
}

#[test]
fn container_literal_typing_rejects_mixed_element_families() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] bad(): vec[int] = {\n\
             return {1, false};\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("container element expects")
        }),
        "Expected a mixed-container-element diagnostic, got: {errors:?}"
    );
}

#[test]
fn container_literal_typing_accepts_set_and_map_contexts() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] tuple_second(): str = {\n\
             var parts: set[int, str] = {1, \"two\"};\n\
             return parts[1];\n\
         }\n\
         fun[] lookup(): int = {\n\
             var counts: map[str, int] = {{\"US\", 45}, {\"DE\", 82}};\n\
             return counts[\"US\"];\n\
         }\n",
    )]);

    let (_parts_id, parts) = find_typed_symbol(&typed, "parts", SymbolKind::ValueBinding);
    let (_counts_id, counts) = find_typed_symbol(&typed, "counts", SymbolKind::ValueBinding);
    let tuple_second = find_named_routine_syntax_id(&typed, "tuple_second");
    let lookup = find_named_routine_syntax_id(&typed, "lookup");

    assert!(matches!(
        typed.type_table().get(parts.declared_type.expect("set binding should lower")),
        Some(CheckedType::Set { member_types })
            if member_types == &vec![typed.builtin_types().int, typed.builtin_types().str_]
    ));
    assert!(matches!(
        typed.type_table().get(counts.declared_type.expect("map binding should lower")),
        Some(CheckedType::Map { key_type, value_type })
            if *key_type == typed.builtin_types().str_
                && *value_type == typed.builtin_types().int
    ));
    assert_eq!(
        typed
            .typed_node(tuple_second)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Str))
    );
    assert_eq!(
        typed
            .typed_node(lookup)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn container_literal_typing_rejects_bad_map_pairs_and_nonliteral_heterogeneous_set_indexes() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] bad_map(): map[str, int] = {\n\
             return {{1, 45}};\n\
         }\n\
         fun[] bad_set(idx: int): int = {\n\
             var parts: set[int, str] = {1, \"two\"};\n\
             return parts[idx];\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("map key expects")
        }),
        "Expected a map-key compatibility diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("heterogeneous sets is not part of the V1 typecheck milestone")
        }),
        "Expected a heterogeneous-set indexing diagnostic, got: {errors:?}"
    );
}

#[test]
fn record_initializer_typing_accepts_nested_record_construction() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ Bonus: rec = {\n\
             hta: int;\n\
             ra: int\n\
         }\n\
         typ Salary: rec = {\n\
             basic: int;\n\
             bonus: Bonus\n\
         }\n\
         typ Employee: rec = {\n\
             name: str;\n\
             salary: Salary\n\
         }\n\
         fun[] build(): Employee = {\n\
             return {\n\
                 name = \"Mark\",\n\
                 salary = {\n\
                     basic = 15000,\n\
                     bonus = { hta = 2100, ra = 5000 },\n\
                 },\n\
             };\n\
         }\n",
    )]);

    let (employee_id, _employee) = find_typed_symbol(&typed, "Employee", SymbolKind::Type);
    let syntax_id = find_named_routine_syntax_id(&typed, "build");

    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Declared {
            symbol: employee_id,
            name: "Employee".to_string(),
            kind: DeclaredTypeKind::Type,
        })
    );
}

#[test]
fn record_initializer_typing_accepts_named_binding_and_call_argument_contexts() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ User: rec = {\n\
             name: str;\n\
             count: int\n\
         }\n\
         fun[] count(user: User): int = {\n\
             return user.count;\n\
         }\n\
         fun[] build(): int = {\n\
             var current: User = { name = \"ok\", count = 1 };\n\
             return count({ name = \"next\", count = 2 });\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "build");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn record_initializer_typing_rejects_missing_unknown_and_mismatched_fields() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "typ User: rec = {\n\
             name: str;\n\
             count: int\n\
         }\n\
         fun[] bad_type(): User = {\n\
             return { name = false, count = 1 };\n\
         }\n\
         fun[] bad_field(): User = {\n\
             return { name = \"ok\" };\n\
         }\n\
         fun[] unknown_field(): User = {\n\
             return { name = \"ok\", count = 1, extra = 3 };\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("record field 'name' expects")
        }),
        "Expected a mismatched record-field diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("missing required fields: count")
        }),
        "Expected a missing-record-field diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("does not define a field named 'extra'")
        }),
        "Expected an unknown-record-field diagnostic, got: {errors:?}"
    );
}

#[test]
fn workspace_record_initializer_typing_accepts_imported_named_record_contexts() {
    let root = unique_temp_dir("workspace_imported_record_initializers");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] User: rec = {\n",
                    "    name: str;\n",
                    "    count: int;\n",
                    "}\n",
                    "fun[exp] count(user: User): int = {\n",
                    "    return user.count;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "var imported_user: User = { name = \"ok\", count = 1 };\n",
                    "fun[] main(): int = {\n",
                    "    return count({ name = \"next\", count = 2 });\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept imported named record initializers in bindings and call arguments");
    let syntax_id = find_named_routine_syntax_id(&typed, "main");

    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn entry_value_typing_accepts_entry_variant_accesses() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ Color: ent = {\n\
             var BLUE: str = \"#0037cd\";\n\
             var RED: str = \"#ff0000\";\n\
         }\n\
         fun[] blue(): str = {\n\
             return Color.BLUE;\n\
         }\n",
    )]);

    let syntax_id = find_named_routine_syntax_id(&typed, "blue");
    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Str))
    );
}

#[test]
fn entry_value_typing_accepts_named_entry_binding_return_and_call_contexts() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ Status: ent = {\n\
             var OK: int = 1;\n\
             var FAIL: int = 2;\n\
         }\n\
         fun[] echo(status: Status): Status = {\n\
             return status;\n\
         }\n\
         fun[] main(): Status = {\n\
             var current: Status = Status.OK;\n\
             return echo(Status.FAIL);\n\
         }\n",
    )]);

    let (status_id, _status) = find_typed_symbol(&typed, "Status", SymbolKind::Type);
    let syntax_id = find_named_routine_syntax_id(&typed, "main");

    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Declared {
            symbol: status_id,
            name: "Status".to_string(),
            kind: DeclaredTypeKind::Type,
        })
    );
}

#[test]
fn entry_value_typing_rejects_unknown_variants() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "typ Color: ent = {\n\
             var BLUE: str = \"#0037cd\";\n\
         }\n\
         fun[] bad(): str = {\n\
             return Color.BLACK;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("entry receiver does not expose a variant named 'BLACK'")
        }),
        "Expected an unknown-entry-variant diagnostic, got: {errors:?}"
    );
}

#[test]
fn workspace_entry_value_typing_accepts_imported_named_entry_contexts() {
    let root = unique_temp_dir("workspace_imported_entry_values");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] Status: ent = {\n",
                    "    var OK: int = 1;\n",
                    "    var FAIL: int = 2;\n",
                    "}\n",
                    "fun[exp] echo(status: Status): Status = {\n",
                    "    return status;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "var imported_status: Status = Status.OK;\n",
                    "fun[] main(): Status = {\n",
                    "    return echo(Status.FAIL);\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept imported named entry values in bindings, returns, and call arguments");
    let (_status_id, status) = find_typed_symbol(&typed, "Status", SymbolKind::Type);
    let syntax_id = find_named_routine_syntax_id(&typed, "main");

    assert_eq!(
        typed.type_table().get(status.declared_type.expect("imported entry type should retain a declared shell")),
        Some(&CheckedType::Entry {
            variants: BTreeMap::from([
                ("FAIL".to_string(), Some(typed.builtin_types().int)),
                ("OK".to_string(), Some(typed.builtin_types().int)),
            ]),
        })
    );
    assert!(matches!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(CheckedType::Declared {
            name,
            kind: DeclaredTypeKind::Type,
            ..
        }) if name == "Status"
    ));
}

#[test]
fn workspace_aggregate_typing_keeps_qualified_imported_record_and_entry_surfaces() {
    let root = unique_temp_dir("workspace_qualified_imported_aggregates");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] User: rec = {\n",
                    "    count: int;\n",
                    "}\n",
                    "var[exp] current: User = { count = 1 };\n",
                    "typ[exp] Status: ent = {\n",
                    "    var OK: int = 1;\n",
                    "    var FAIL: int = 2;\n",
                    "}\n",
                    "fun[exp] echo(status: Status): Status = {\n",
                    "    return status;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): Status = {\n",
                    "    var total: int = shared::current.count;\n",
                    "    return shared::echo(shared::Status.OK);\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should keep qualified imported record and entry aggregate surfaces");
    let syntax_id = find_named_routine_syntax_id(&typed, "main");

    assert!(matches!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(CheckedType::Declared {
            name,
            kind: DeclaredTypeKind::Type,
            ..
        }) if name == "Status"
    ));
}

#[test]
fn shell_typing_accepts_optional_and_error_payload_lifting() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "ali MaybeText: opt[str]\n\
         ali Failure: err[str]\n\
         var label: MaybeText = \"ok\"\n\
         var issue: Failure = \"bad\"\n\
         fun[] maybe(): MaybeText = {\n\
             return \"ready\";\n\
         }\n\
         fun[] fail(): int: Failure = {\n\
             report \"broken\";\n\
         }\n",
    )]);

    let (maybe_id, _maybe_alias) = find_typed_symbol(&typed, "MaybeText", SymbolKind::Alias);
    let (failure_id, _failure_alias) = find_typed_symbol(&typed, "Failure", SymbolKind::Alias);
    let (_label_id, label) = find_typed_symbol(&typed, "label", SymbolKind::ValueBinding);
    let (_issue_id, issue) = find_typed_symbol(&typed, "issue", SymbolKind::ValueBinding);
    let maybe_syntax = find_named_routine_syntax_id(&typed, "maybe");
    let fail_syntax = find_named_routine_syntax_id(&typed, "fail");

    assert_eq!(
        typed.type_table().get(label.declared_type.expect("optional binding should lower")),
        Some(&CheckedType::Declared {
            symbol: maybe_id,
            name: "MaybeText".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    assert_eq!(
        typed.type_table().get(issue.declared_type.expect("error binding should lower")),
        Some(&CheckedType::Declared {
            symbol: failure_id,
            name: "Failure".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    assert_eq!(
        typed
            .typed_node(maybe_syntax)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Declared {
            symbol: maybe_id,
            name: "MaybeText".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    assert_eq!(
        typed
            .typed_node(fail_syntax)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn shell_typing_rejects_mismatched_optional_payloads() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "ali MaybeText: opt[str]\n\
         fun[] bad(): int = {\n\
             var label: MaybeText = 1;\n\
             return 0;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error
                    .message()
                    .contains("initializer for 'label' expects")
        }),
        "Expected an optional-shell mismatch diagnostic, got: {errors:?}"
    );
}

#[test]
fn shell_typing_rejects_pointer_surfaces_as_v3_only() {
    let errors = typecheck_fixture_folder_errors(&[("main.fol", "ali CounterPtr: ptr[int]\n")]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("pointer types are part of the V3 systems milestone")
        }),
        "Expected a V3 pointer-boundary diagnostic, got: {errors:?}"
    );
}

#[test]
fn operator_typing_accepts_v1_scalar_operator_families() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] math(): int = {\n\
             return 1 + 2 * 3 - 4 % 2;\n\
         }\n\
         fun[] text(): str = {\n\
             return \"ab\" + \"cd\";\n\
         }\n\
         fun[] compare(): bol = {\n\
             return 1 < 2 and 3 != 4;\n\
         }\n\
         fun[] invert(flag: bol): bol = {\n\
             return not flag xor false;\n\
         }\n",
    )]);

    for (name, expected) in [
        ("math", CheckedType::Builtin(BuiltinType::Int)),
        ("text", CheckedType::Builtin(BuiltinType::Str)),
        ("compare", CheckedType::Builtin(BuiltinType::Bool)),
        ("invert", CheckedType::Builtin(BuiltinType::Bool)),
    ] {
        let syntax_id = find_named_routine_syntax_id(&typed, name);
        assert_eq!(
            typed
                .typed_node(syntax_id)
                .and_then(|node| node.inferred_type)
                .and_then(|type_id| typed.type_table().get(type_id)),
            Some(&expected),
            "Expected {name} to typecheck as {expected:?}"
        );
    }
}

#[test]
fn operator_typing_rejects_invalid_scalar_pairs_and_pointer_operators() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] bad_math(): int = {\n\
             return true + 1;\n\
         }\n\
         fun[] bad_logic(): bol = {\n\
             return 1 and 2;\n\
         }\n\
         fun[] bad_ref(value: int): int = {\n\
             return &value;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error.message().contains("binary operator 'Add' is not valid")
        }),
        "Expected an invalid arithmetic-operator diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error.message().contains("binary operator 'And' is not valid")
        }),
        "Expected an invalid logical-operator diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("pointer operators are part of the V3 systems milestone")
        }),
        "Expected a pointer-operator boundary diagnostic, got: {errors:?}"
    );
}

#[test]
fn coercion_policy_rejects_implicit_int_float_cross_family_conversions() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] take_float(value: flt): flt = {\n\
             return value;\n\
         }\n\
         fun[] bad_binding(): int = {\n\
             var count: int = 1.5;\n\
             return count;\n\
         }\n\
         fun[] bad_call(): flt = {\n\
             return take_float(1);\n\
         }\n\
         fun[] bad_return(): int = {\n\
             return 1.5;\n\
         }\n\
         fun[] bad_report(): int : int = {\n\
             report 1.5;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error
                    .message()
                    .contains("initializer for 'count' expects")
        }),
        "Expected an initializer coercion diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("call to 'take_float' expects")
        }),
        "Expected a call coercion diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("return expects")
        }),
        "Expected a return coercion diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error.message().contains("report expects")
        }),
        "Expected a report coercion diagnostic, got: {errors:?}"
    );
}

#[test]
fn cast_policy_rejects_as_and_cast_surfaces_in_v1() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "var text: str = \"label\"\n\
         var target: int = 0\n\
         fun[] bad_as(value: int): int = {\n\
             return value as text;\n\
         }\n\
         fun[] bad_cast(value: int): int = {\n\
             return value cast target;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("explicit 'as' casts are not part of the V1 typecheck milestone")
                && error.diagnostic_location().is_some()
        }),
        "Expected an unsupported 'as' cast diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("explicit 'cast' operators are not part of the V1 typecheck milestone")
                && error.diagnostic_location().is_some()
        }),
        "Expected an unsupported 'cast' diagnostic, got: {errors:?}"
    );
}

#[test]
fn literal_family_policy_accepts_matching_integer_and_float_sites() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] take_int(value: int): int = {\n\
             return value;\n\
         }\n\
         fun[] take_float(value: flt): flt = {\n\
             return value;\n\
         }\n\
         fun[] good_int(): int = {\n\
             var count: int = 1;\n\
             return take_int(2);\n\
         }\n\
         fun[] good_float(): flt = {\n\
             var ratio: flt = 1.5;\n\
             return take_float(2.5);\n\
         }\n",
    )]);

    let (_count_id, count) = find_typed_symbol(&typed, "count", SymbolKind::ValueBinding);
    let (_ratio_id, ratio) = find_typed_symbol(&typed, "ratio", SymbolKind::ValueBinding);
    let good_int = find_named_routine_syntax_id(&typed, "good_int");
    let good_float = find_named_routine_syntax_id(&typed, "good_float");

    assert_eq!(
        typed.type_table().get(count.declared_type.expect("int literal binding should lower")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
    assert_eq!(
        typed.type_table().get(ratio.declared_type.expect("float literal binding should lower")),
        Some(&CheckedType::Builtin(BuiltinType::Float))
    );
    assert_eq!(
        typed
            .typed_node(good_int)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
    assert_eq!(
        typed
            .typed_node(good_float)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Float))
    );
}

#[test]
fn v1_boundary_rejects_generic_headers_and_meta_declarations() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
         "fun demo(T)(value: int): int = {\n\
             return value;\n\
         }\n\
         typ Bound: rec = {\n\
         }\n\
         typ Box(T: Bound): rec = {\n\
             value: int\n\
         }\n\
         def helper: mod = {\n\
         }\n\
         seg core: mod = {\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("generic routine semantics are not part of the V1 typecheck milestone")
        }),
        "Expected a generic-routine boundary diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("generic type semantics are not part of the V1 typecheck milestone")
        }),
        "Expected a generic-type boundary diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("definition/meta declarations are part of the V2 language milestone")
        }),
        "Expected a def/meta boundary diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("segment/meta declarations are part of the V2 language milestone")
        }),
        "Expected a seg/meta boundary diagnostic, got: {errors:?}"
    );
}

#[test]
fn v1_boundary_rejects_contract_and_conformance_surfaces() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "typ geo: rec = {\n\
         }\n\
         typ Shape(geo): rec[] = {\n\
             value: int\n\
         }\n\
         typ[ext] StrExt: str\n\
         typ Box: rec = {\n\
         }\n\
         imp Self: Box = {\n\
             fun ready(): bol = {\n\
                 return true;\n\
             }\n\
         }\n\
         std geometry: blu = {\n\
             var width: int;\n\
         };\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("type contract conformance is part of the V2 language milestone")
        }),
        "Expected a type-contract boundary diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("type extension declarations are part of the V2 language milestone")
        }),
        "Expected a type-extension boundary diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("implementation declarations are part of the V2 language milestone")
        }),
        "Expected an implementation boundary diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("blueprint standards are part of the V2 language milestone")
        }),
        "Expected a blueprint-standard boundary diagnostic, got: {errors:?}"
    );
}

#[test]
fn v1_boundary_rejects_v3_declaration_surfaces() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "!var cached: int = 1\n\
         ?var watching: int = 1\n\
         @var fresh: int = 1\n\
         var[bor] borrowed: int = 1\n\
         ali Bus: chn[int]\n\
         fun hold(((meshes)): vec[int]): int = {\n\
             return 0;\n\
         }\n\
         ",
    )]);

    for expected in [
        "static binding semantics are not part of the V1 typecheck milestone",
        "reactive binding semantics are not part of the V1 typecheck milestone",
        "heap/new binding semantics are part of the V3 systems milestone",
        "borrowing binding semantics are part of the V3 systems milestone",
        "channel types are not part of the V1 typecheck milestone",
        "mutex parameter semantics are part of the V3 systems milestone",
    ] {
        assert!(
            errors.iter().any(|error| {
                error.kind() == TypecheckErrorKind::Unsupported
                    && error.message().contains(expected)
            }),
            "Expected a V3 declaration boundary diagnostic containing '{expected}', got: {errors:?}"
        );
    }
}

#[test]
fn v1_boundary_rejects_v3_expression_surfaces() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun rangeDemo(): int = {\n\
             var span: int = 0;\n\
             span = 1..2;\n\
             return 0;\n\
         }\n\
         fun channelDemo(value: int): int = {\n\
             var recv: int = 0;\n\
             recv = value[rx];\n\
             return 0;\n\
         }\n\
         fun spawnDemo(value: int): int = {\n\
             var task: int = 0;\n\
             task = [>]value;\n\
             return 0;\n\
         }\n\
         fun asyncDemo(value: int): int = {\n\
             var next: int = 0;\n\
             next = value | async;\n\
             return 0;\n\
         }\n\
         fun awaitDemo(value: int): int = {\n\
             var next: int = 0;\n\
             next = value | await;\n\
             return 0;\n\
         }\n\
         pro selectDemo(value: int): int = {\n\
             select(value) {\n\
                 return 0;\n\
             }\n\
         }\n\
         fun anonDemo(): int = {\n\
             var worker: int = 0;\n\
             worker = fun(((locks)): vec[int]): int = {\n\
                 return 0;\n\
             };\n\
             return 0;\n\
         }\n",
    )]);

    for expected in [
        "range expressions are not part of the V1 typecheck milestone",
        "channel endpoint access is part of the V3 systems milestone",
        "coroutine spawn expressions are part of the V3 systems milestone",
        "async pipe stages are part of the V3 systems milestone",
        "await pipe stages are part of the V3 systems milestone",
        "select/channel semantics are part of the V3 systems milestone",
        "mutex parameter semantics are part of the V3 systems milestone",
    ] {
        assert!(
            errors.iter().any(|error| {
                error.kind() == TypecheckErrorKind::Unsupported
                    && error.message().contains(expected)
            }),
            "Expected a V3 expression boundary diagnostic containing '{expected}', got: {errors:?}"
        );
    }
}

#[test]
fn ordinary_typechecking_rejects_build_fol_source_units() {
    let errors = typecheck_fixture_folder_errors(&[(
        "build.fol",
        "`package build`\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("ordinary typechecking does not interpret build.fol package semantics")
        }),
        "Expected a build.fol typechecking boundary diagnostic, got: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| {
            error
                .origin()
                .and_then(|origin| origin.file.as_deref())
                .is_some_and(|file| file.ends_with("build.fol"))
        }),
        "Expected build.fol boundary diagnostics to keep the source-unit path, got: {errors:?}"
    );
}

#[test]
fn workspace_typechecking_caches_loaded_packages_by_identity() {
    let root = unique_temp_dir("workspace_typecheck_cache");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("shared/lib.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace typechecking should accept packages that do not yet use imported values");

    assert_eq!(typed.package_count(), 2);
    assert_eq!(typed.entry_program().package_name(), "app");
    assert!(
        typed
            .packages()
            .any(|package| package.identity.display_name == "shared"),
        "Typed workspace should retain typed facts for directly loaded packages"
    );
}

#[test]
fn workspace_typechecking_dedupes_repeated_loaded_packages() {
    let root = unique_temp_dir("workspace_typecheck_dedupe");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("shared/lib.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                concat!(
                    "use left: loc = {\"../shared\"};\n",
                    "use right: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    return 0;\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace typechecking should reuse one typed package per identity");

    assert_eq!(typed.package_count(), 2);
    assert_eq!(
        typed
            .packages()
            .filter(|package| package.identity.display_name == "shared")
            .count(),
        1,
        "Typed workspace should cache one typed package fact per package identity"
    );
}

#[test]
fn workspace_typechecking_imports_mounted_value_and_routine_types_from_foreign_packages() {
    let root = unique_temp_dir("workspace_typecheck_mounted_types");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "var[exp] answer: int = 42;\n",
                    "fun[exp] bump(value: int): int = {\n",
                    "    return value + 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace typechecking should import mounted symbol facts from dependency packages");
    let entry = typed.entry_program();

    let (_answer_id, answer) = find_typed_symbol(entry, "answer", SymbolKind::ValueBinding);
    assert_eq!(
        entry
            .type_table()
            .get(answer.declared_type.expect("mounted imported values should keep translated types")),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );

    let (_bump_id, bump) = find_typed_symbol(entry, "bump", SymbolKind::Routine);
    assert_eq!(
        entry
            .type_table()
            .get(bump.declared_type.expect("mounted imported routines should keep translated signatures")),
        Some(&CheckedType::Routine(RoutineType {
            params: vec![entry.builtin_types().int],
            return_type: Some(entry.builtin_types().int),
            error_type: None,
        }))
    );
}

#[test]
fn workspace_typechecking_preserves_local_only_success_shape() {
    let root = unique_temp_dir("workspace_typecheck_local_only");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[(
            "app/main.fol",
            concat!(
                "ali Count: int\n",
                "fun[] helper(value: Count): Count = {\n",
                "    return value;\n",
                "}\n",
            ),
        )],
    );

    let direct = typecheck_fixture_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Direct typechecking should still accept local-only packages");
    let workspace = typecheck_fixture_workspace_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace typechecking should preserve local-only packages");
    let workspace_entry = workspace.entry_program();

    assert_eq!(workspace.package_count(), 1);

    let (direct_count_id, _direct_count) = find_typed_symbol(&direct, "Count", SymbolKind::Alias);
    let (workspace_count_id, _workspace_count) =
        find_typed_symbol(workspace_entry, "Count", SymbolKind::Alias);
    let (_direct_helper_id, direct_helper) =
        find_typed_symbol(&direct, "helper", SymbolKind::Routine);
    let (_workspace_helper_id, workspace_helper) =
        find_typed_symbol(workspace_entry, "helper", SymbolKind::Routine);

    let direct_signature = match direct
        .type_table()
        .get(direct_helper.declared_type.expect("direct helper should have a signature"))
    {
        Some(CheckedType::Routine(signature)) => signature,
        other => panic!("expected direct helper routine signature, got {other:?}"),
    };
    assert_eq!(direct_signature.params.len(), 1);
    assert_eq!(direct_signature.error_type, None);
    assert_eq!(
        direct.type_table().get(direct_signature.params[0]),
        Some(&CheckedType::Declared {
            symbol: direct_count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    assert_eq!(
        direct_signature
            .return_type
            .and_then(|type_id| direct.type_table().get(type_id)),
        Some(&CheckedType::Declared {
            symbol: direct_count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );

    let workspace_signature = match workspace_entry.type_table().get(
        workspace_helper
            .declared_type
            .expect("workspace helper should have a signature"),
    ) {
        Some(CheckedType::Routine(signature)) => signature,
        other => panic!("expected workspace helper routine signature, got {other:?}"),
    };
    assert_eq!(workspace_signature.params.len(), 1);
    assert_eq!(workspace_signature.error_type, None);
    assert_eq!(
        workspace_entry
            .type_table()
            .get(workspace_signature.params[0]),
        Some(&CheckedType::Declared {
            symbol: workspace_count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    assert_eq!(
        workspace_signature
            .return_type
            .and_then(|type_id| workspace_entry.type_table().get(type_id)),
        Some(&CheckedType::Declared {
            symbol: workspace_count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
}

#[test]
fn workspace_typechecking_keeps_loaded_package_declaration_signatures() {
    let root = unique_temp_dir("workspace_typecheck_loaded_package_decls");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "ali Count: int\n",
                    "var[exp] answer: Count = 42;\n",
                    "fun[exp] bump(value: Count): Count = {\n",
                    "    return value + 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace typechecking should type loaded package declarations before entry typing");
    let shared = &find_typed_package(&typed, "shared").program;

    let (count_id, _count) = find_typed_symbol(shared, "Count", SymbolKind::Alias);
    let (_answer_id, answer) = find_typed_symbol(shared, "answer", SymbolKind::ValueBinding);
    let (_bump_id, bump) = find_typed_symbol(shared, "bump", SymbolKind::Routine);

    assert_eq!(
        shared
            .type_table()
            .get(answer.declared_type.expect("loaded package values should lower declared types")),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    let signature = match shared
        .type_table()
        .get(bump.declared_type.expect("loaded package routines should lower signatures"))
    {
        Some(CheckedType::Routine(signature)) => signature,
        other => panic!("expected loaded package routine signature, got {other:?}"),
    };
    assert_eq!(signature.params.len(), 1);
    assert_eq!(signature.error_type, None);
    assert_eq!(
        shared.type_table().get(signature.params[0]),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
    assert_eq!(
        signature
            .return_type
            .and_then(|type_id| shared.type_table().get(type_id)),
        Some(&CheckedType::Declared {
            symbol: count_id,
            name: "Count".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
}

#[test]
fn workspace_expression_typing_keeps_plain_imported_value_reference_types() {
    let root = unique_temp_dir("workspace_imported_value_reference_types");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("shared/lib.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    return answer;\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept imported value references");
    let reference = find_typed_reference(&typed, "answer", ReferenceKind::Identifier);

    assert_eq!(
        typed.type_table().get(
            reference
                .resolved_type
                .expect("imported value references should keep a resolved type"),
        ),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn workspace_expression_typing_keeps_plain_imported_call_types() {
    let root = unique_temp_dir("workspace_imported_call_types");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "fun[exp] answer(): int = {\n",
                    "    return 42;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    return answer();\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept imported routine calls");
    let reference = find_typed_reference(&typed, "answer", ReferenceKind::FunctionCall);

    assert_eq!(
        typed.type_table().get(
            reference
                .resolved_type
                .expect("imported call references should keep a resolved type"),
        ),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn workspace_typechecking_imports_alias_record_and_entry_type_facts() {
    let root = unique_temp_dir("workspace_imported_type_facts");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] Count: int;\n",
                    "typ[exp] Point: rec = {\n",
                    "    x: int;\n",
                    "    y: int;\n",
                    "}\n",
                    "typ[exp] Outcome: ent = {\n",
                    "    var Ok: int = 1;\n",
                    "    con Fail: str = \"bad\";\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should import semantic type facts for exported type surfaces");

    let (_count_id, count) = find_typed_symbol(&typed, "Count", SymbolKind::Type);
    let (_point_id, point) = find_typed_symbol(&typed, "Point", SymbolKind::Type);
    let (_outcome_id, outcome) = find_typed_symbol(&typed, "Outcome", SymbolKind::Type);

    assert_eq!(
        typed.type_table().get(
            count
                .declared_type
                .expect("imported aliases should keep lowered semantic types"),
        ),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );

    let point_fields = BTreeMap::from([
        ("x".to_string(), typed.builtin_types().int),
        ("y".to_string(), typed.builtin_types().int),
    ]);
    assert_eq!(
        typed.type_table().get(
            point
                .declared_type
                .expect("imported record types should keep lowered semantic types"),
        ),
        Some(&CheckedType::Record { fields: point_fields })
    );

    let outcome_variants = BTreeMap::from([
        ("Fail".to_string(), Some(typed.builtin_types().str_)),
        ("Ok".to_string(), Some(typed.builtin_types().int)),
    ]);
    assert_eq!(
        typed.type_table().get(
            outcome
                .declared_type
                .expect("imported entry types should keep lowered semantic types"),
        ),
        Some(&CheckedType::Entry {
            variants: outcome_variants,
        })
    );
}

#[test]
fn workspace_typechecking_keeps_direct_loc_import_declaration_facts() {
    let root = unique_temp_dir("workspace_direct_loc_decls");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] Count: int;\n",
                    "var[exp] answer: Count = 42;\n",
                    "fun[exp] bump(value: Count): Count = {\n",
                    "    return value + 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should keep direct loc import declaration facts");

    assert_imported_declared_count_binding_and_routine(&typed, SymbolKind::Type);
}

#[test]
fn workspace_typechecking_keeps_direct_std_import_declaration_facts() {
    let root = unique_temp_dir("workspace_direct_std_decls");
    let std_root = root.join("std");
    create_dir_all(&std_root).expect("Std root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "std/fmt/lib.fol",
                concat!(
                    "typ[exp] Count: int;\n",
                    "var[exp] answer: Count = 42;\n",
                    "fun[exp] bump(value: Count): Count = {\n",
                    "    return value + 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                "use fmt: std = {fmt};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(
        &root,
        "app",
        ResolverConfig {
            std_root: Some(
                std_root
                    .to_str()
                    .expect("Std root should be valid UTF-8")
                    .to_string(),
            ),
            package_store_root: None,
        },
    )
    .expect("Workspace entry typing should keep direct std import declaration facts");

    assert_imported_declared_count_binding_and_routine(&typed, SymbolKind::Type);
}

#[test]
fn workspace_typechecking_keeps_direct_pkg_import_declaration_facts() {
    let root = unique_temp_dir("workspace_direct_pkg_decls");
    let store_root = root.join("store");
    create_dir_all(&store_root).expect("Package store root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("store/json/package.yaml", "name: json\nversion: 1.0.0\n"),
            ("store/json/build.fol", "def root: loc = \"src\";\n"),
            (
                "store/json/src/lib.fol",
                concat!(
                    "typ[exp] Count: int;\n",
                    "var[exp] answer: Count = 42;\n",
                    "fun[exp] bump(value: Count): Count = {\n",
                    "    return value + 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                "use json: pkg = {json};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(
        &root,
        "app",
        ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("Package store root should be valid UTF-8")
                    .to_string(),
            ),
        },
    )
    .expect("Workspace entry typing should keep direct pkg import declaration facts");

    assert_imported_declared_count_binding_and_routine(&typed, SymbolKind::Type);
}

#[test]
fn workspace_typechecking_keeps_transitive_pkg_import_declaration_facts() {
    let root = unique_temp_dir("workspace_transitive_pkg_decls");
    let store_root = root.join("store");
    create_dir_all(&store_root).expect("Package store root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("store/core/package.yaml", "name: core\nversion: 1.0.0\n"),
            ("store/core/build.fol", "def root: loc = \"src\";\n"),
            ("store/core/src/lib.fol", "typ[exp] Count: int;\n"),
            ("store/json/package.yaml", "name: json\nversion: 1.0.0\n"),
            (
                "store/json/build.fol",
                concat!(
                    "def core: pkg = \"core\";\n",
                    "def root: loc = \"src\";\n",
                ),
            ),
            (
                "store/json/src/lib.fol",
                concat!(
                    "use core: pkg = {core};\n",
                    "var[exp] answer: Count = 42;\n",
                    "fun[exp] bump(value: Count): Count = {\n",
                    "    return value + 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                "use json: pkg = {json};\nfun[] main(): int = {\n    return 0;\n}\n",
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(
        &root,
        "app",
        ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("Package store root should be valid UTF-8")
                    .to_string(),
            ),
        },
    )
    .expect("Workspace entry typing should keep transitive pkg declaration facts");

    let (_answer_id, answer) = find_typed_symbol(&typed, "answer", SymbolKind::ValueBinding);
    let (_bump_id, bump) = find_typed_symbol(&typed, "bump", SymbolKind::Routine);

    assert!(matches!(
        typed
            .type_table()
            .get(answer.declared_type.expect("transitive imported values should keep declared types")),
        Some(CheckedType::Declared {
            name,
            kind: DeclaredTypeKind::Type,
            ..
        }) if name == "Count"
    ));

    let signature = match typed.type_table().get(
        bump.declared_type
            .expect("transitive imported routines should keep translated signatures"),
    ) {
        Some(CheckedType::Routine(signature)) => signature,
        other => panic!("expected transitive imported routine signature, got {other:?}"),
    };
    assert!(matches!(
        typed.type_table().get(signature.params[0]),
        Some(CheckedType::Declared {
            name,
            kind: DeclaredTypeKind::Type,
            ..
        }) if name == "Count"
    ));
    assert!(matches!(
        signature
            .return_type
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(CheckedType::Declared {
            name,
            kind: DeclaredTypeKind::Type,
            ..
        }) if name == "Count"
    ));
}

#[test]
fn workspace_expression_typing_keeps_plain_imported_value_types_in_bindings_returns_and_call_args() {
    let root = unique_temp_dir("workspace_imported_value_contexts");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("shared/lib.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] echo(value: int): int = {\n",
                    "    return value;\n",
                    "}\n",
                    "fun[] main(): int = {\n",
                    "    var current: int = answer;\n",
                    "    var echoed: int = echo(answer);\n",
                    "    return answer;\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept plain imported values in all basic expression contexts");
    let references = find_typed_references(&typed, "answer", ReferenceKind::Identifier);

    assert_eq!(references.len(), 3, "expected imported value references in binding, call argument, and return");
    for reference in references {
        assert_eq!(
            typed.type_table().get(
                reference
                    .resolved_type
                    .expect("imported value references should keep a resolved type"),
            ),
            Some(&CheckedType::Builtin(BuiltinType::Int))
        );
    }
}

#[test]
fn workspace_expression_typing_rejects_plain_imported_call_argument_mismatches() {
    let root = unique_temp_dir("workspace_imported_call_checks");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                "fun[exp] emit(value: int): int = {\n    return value;\n}\n",
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    return emit(\"bad\");\n",
                    "}\n",
                ),
            ),
        ],
    );

    let errors = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect_err("Workspace entry typing should reject imported call argument mismatches");

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::IncompatibleType
                && error
                    .message()
                    .contains("call to 'emit' expects 'Builtin(Int)'")
        }),
        "Expected imported-call argument mismatch diagnostic, got: {errors:?}"
    );
}

#[test]
fn workspace_expression_typing_keeps_qualified_imported_value_and_call_types() {
    let root = unique_temp_dir("workspace_qualified_import_types");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "var[exp] answer: int = 42;\n",
                    "fun[exp] emit(value: int): int = {\n",
                    "    return value;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    var current: int = shared::answer;\n",
                    "    return shared::emit(current);\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept qualified imported value and call references");
    let value_reference =
        find_typed_reference(&typed, "shared::answer", ReferenceKind::QualifiedIdentifier);
    let call_reference =
        find_typed_reference(&typed, "shared::emit", ReferenceKind::QualifiedFunctionCall);

    assert_eq!(
        typed.type_table().get(
            value_reference
                .resolved_type
                .expect("qualified imported value references should keep a resolved type"),
        ),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
    assert_eq!(
        typed.type_table().get(
            call_reference
                .resolved_type
                .expect("qualified imported call references should keep a resolved type"),
        ),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn workspace_expression_typing_types_plain_imported_method_calls() {
    let root = unique_temp_dir("workspace_imported_method_calls");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] Counter: rec = {\n",
                    "    value: int;\n",
                    "}\n",
                    "var[exp] current: Counter;\n",
                    "fun[exp] (Counter)read(): int = {\n",
                    "    return 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    return current.read();\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept imported method calls through typed package facts");
    let syntax_id = find_named_routine_syntax_id(&typed, "main");

    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn workspace_expression_typing_types_qualified_imported_method_calls() {
    let root = unique_temp_dir("workspace_qualified_imported_method_calls");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] Counter: rec = {\n",
                    "    value: int;\n",
                    "}\n",
                    "var[exp] current: Counter;\n",
                    "fun[exp] (Counter)read(): int = {\n",
                    "    return 1;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    return shared::current.read();\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should accept qualified imported method calls through typed package facts");
    let syntax_id = find_named_routine_syntax_id(&typed, "main");

    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn workspace_expression_typing_expands_imported_alias_record_shells_for_field_access() {
    let root = unique_temp_dir("workspace_imported_alias_record_field_access");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                concat!(
                    "typ CounterShape: rec = {\n",
                    "    value: int;\n",
                    "}\n",
                    "ali Counter: CounterShape\n",
                    "var[exp] current: Counter = { value = 1 }\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] main(): int = {\n",
                    "    return current.value;\n",
                    "}\n",
                ),
            ),
        ],
    );

    let typed = typecheck_fixture_workspace_entry_with_config(&root, "app", ResolverConfig::default())
        .expect("Workspace entry typing should expand imported alias record shells for field access");
    let syntax_id = find_named_routine_syntax_id(&typed, "main");

    assert_eq!(
        typed
            .typed_node(syntax_id)
            .and_then(|node| node.inferred_type)
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
}

#[test]
fn reopened_v1_blocker_loc_imported_values_still_fail_typecheck() {
    let root = unique_temp_dir("reopened_loc_import");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("shared/lib.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer;\n}\n",
            ),
        ],
    );

    let errors = typecheck_fixture_entry_with_config(&root, "app", ResolverConfig::default())
        .expect_err("Current head should still fail imported loc value typing");

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("requires workspace-aware typechecking in V1")
                && error.diagnostic_location().is_some()
        }),
        "Expected the explicit loc-import blocker diagnostic, got: {errors:?}"
    );
}

#[test]
fn reopened_v1_blocker_std_imported_values_still_fail_typecheck() {
    let root = unique_temp_dir("reopened_std_import");
    let std_root = root.join("std");
    create_dir_all(&std_root).expect("Std root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("std/fmt/value.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                "use fmt: std = {fmt};\nfun[] main(): int = {\n    return answer;\n}\n",
            ),
        ],
    );

    let errors = typecheck_fixture_entry_with_config(
        &root,
        "app",
        ResolverConfig {
            std_root: Some(
                std_root
                    .to_str()
                    .expect("std fixture path should be utf8")
                    .to_string(),
            ),
            package_store_root: None,
        },
    )
    .expect_err("Current head should still fail imported std value typing");

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("requires workspace-aware typechecking in V1")
                && error.diagnostic_location().is_some()
        }),
        "Expected the explicit std-import blocker diagnostic, got: {errors:?}"
    );
}

#[test]
fn reopened_v1_blocker_pkg_imported_values_still_fail_typecheck() {
    let root = unique_temp_dir("reopened_pkg_import");
    let store_root = root.join("store");
    create_dir_all(&store_root).expect("Package store root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("store/json/package.yaml", "name: json\nversion: 1.0.0\n"),
            ("store/json/build.fol", "def root: loc = \"src\";\n"),
            ("store/json/src/lib.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                "use json: pkg = {json};\nfun[] main(): int = {\n    return answer;\n}\n",
            ),
        ],
    );

    let errors = typecheck_fixture_entry_with_config(
        &root,
        "app",
        ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("package store fixture path should be utf8")
                    .to_string(),
            ),
        },
    )
    .expect_err("Current head should still fail imported pkg value typing");

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("requires workspace-aware typechecking in V1")
                && error.diagnostic_location().is_some()
        }),
        "Expected the explicit pkg-import blocker diagnostic, got: {errors:?}"
    );
}

#[test]
fn reopened_v1_blocker_imported_routine_calls_still_fail_typecheck() {
    let root = unique_temp_dir("reopened_imported_call");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            (
                "shared/lib.fol",
                "fun[exp] answer(): int = {\n    return 42;\n}\n",
            ),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer();\n}\n",
            ),
        ],
    );

    let errors = typecheck_fixture_entry_with_config(&root, "app", ResolverConfig::default())
        .expect_err("Current head should still fail imported routine call typing");

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::Unsupported
                && error
                    .message()
                    .contains("requires workspace-aware typechecking in V1")
                && error.diagnostic_location().is_some()
        }),
        "Expected the explicit imported-call blocker diagnostic, got: {errors:?}"
    );
}

#[test]
fn shell_typing_accepts_nil_in_optional_and_error_binding_contexts() {
    let typed = typecheck_fixture_folder(&[
        (
            "main.fol",
            "ali MaybeText: opt[str]\nali Failure: err[str]\nali BareFailure: err[]\nvar label: MaybeText = nil\nvar raised: Failure = nil\nvar empty: BareFailure = nil\n",
        ),
    ]);

    let (_label_id, label) = find_typed_symbol(&typed, "label", SymbolKind::ValueBinding);
    let (_raised_id, raised) = find_typed_symbol(&typed, "raised", SymbolKind::ValueBinding);
    let (_empty_id, empty) = find_typed_symbol(&typed, "empty", SymbolKind::ValueBinding);

    assert!(matches!(
        typed
            .type_table()
            .get(label.declared_type.expect("label should keep its declared type")),
        Some(CheckedType::Declared { name, .. }) if name == "MaybeText"
    ));
    assert!(matches!(
        typed
            .type_table()
            .get(raised.declared_type.expect("raised should keep its declared type")),
        Some(CheckedType::Declared { name, .. }) if name == "Failure"
    ));
    assert!(matches!(
        typed
            .type_table()
            .get(empty.declared_type.expect("empty should keep its declared type")),
        Some(CheckedType::Declared { name, .. }) if name == "BareFailure"
    ));
}

#[test]
fn typecheck_reports_explicit_top_level_binding_type_requirements() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "var mystery\nfun[] main(): int = {\n    return mystery;\n}\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("binding 'mystery' needs a declared type or an inferable initializer in V1")
        }),
        "Expected the explicit top-level binding type diagnostic, got: {errors:?}"
    );
}

#[test]
fn typecheck_reports_explicit_local_binding_type_requirements() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] main(): int = {\n    var mystery\n    return mystery;\n}\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("binding 'mystery' needs a declared type or an inferable initializer in V1")
        }),
        "Expected the explicit local binding type diagnostic, got: {errors:?}"
    );
}

#[test]
fn reopened_v1_import_failures_no_longer_use_raw_lowered_type_fallbacks() {
    let root = unique_temp_dir("reopened_loc_import_regression");
    create_dir_all(&root).expect("Fixture root should be creatable");
    write_fixture_files(
        &root,
        &[
            ("shared/lib.fol", "var[exp] answer: int = 42;\n"),
            (
                "app/main.fol",
                "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer;\n}\n",
            ),
        ],
    );

    let errors = typecheck_fixture_entry_with_config(&root, "app", ResolverConfig::default())
        .expect_err("Legacy single-program typecheck should still reject imported value typing");

    assert!(
        errors.iter().all(|error| {
            error.kind() != TypecheckErrorKind::Internal
                && !error.message().contains("does not have a lowered type yet")
        }),
        "Imported fallback regressions should no longer surface raw lowered-type wording, got: {errors:?}"
    );
}

#[test]
fn reopened_v1_binding_failures_no_longer_use_raw_lowered_type_fallbacks() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] main(): int = {\n    var mystery\n    return mystery;\n}\n",
    )]);

    assert!(
        errors.iter().all(|error| {
            error.kind() != TypecheckErrorKind::Internal
                && !error.message().contains("does not have a lowered type yet")
        }),
        "Binding fallback regressions should no longer surface raw lowered-type wording, got: {errors:?}"
    );
}

#[test]
fn nil_typing_rejects_missing_expected_shell_contexts() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "var label = nil\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("nil literals require an expected opt[...] or err[...] shell type in V1")
        }),
        "Expected the nil expected-shell diagnostic, got: {errors:?}"
    );
}

#[test]
fn shell_typing_accepts_postfix_unwrap_for_optional_and_typed_error_values() {
    let _typed = typecheck_fixture_folder(&[(
        "main.fol",
        "ali MaybeText: opt[str]\nali Failure: err[str]\nfun[] take_text(value: MaybeText): str = {\n    return value!;\n}\nfun[] take_error(value: Failure): str = {\n    return value!;\n}\n",
    )]);
}

#[test]
fn shell_typing_rejects_postfix_unwrap_for_bare_error_shells() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "ali BareFailure: err[]\nfun[] main(value: BareFailure): str = {\n    return value!;\n}\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("unwrap requires an opt[...] or err[...] shell with a value type in V1")
        }),
        "Expected the unwrap shell diagnostic, got: {errors:?}"
    );
}

#[test]
fn shell_typing_accepts_nil_in_return_and_call_argument_contexts() {
    let _typed = typecheck_fixture_folder(&[(
        "main.fol",
        "ali MaybeText: opt[str]\nfun[] echo(value: MaybeText): MaybeText = {\n    return value;\n}\nfun[] make(): MaybeText = {\n    return nil;\n}\nfun[] main(): MaybeText = {\n    return echo(nil);\n}\n",
    )]);
}

#[test]
fn shell_typing_accepts_postfix_unwrap_in_binding_and_return_contexts() {
    let _typed = typecheck_fixture_folder(&[(
        "main.fol",
        "ali MaybeText: opt[str]\nfun[] main(value: MaybeText): str = {\n    var label: str = value!;\n    return value!;\n}\n",
    )]);
}

#[test]
fn shell_typing_rejects_postfix_unwrap_for_non_shell_targets() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] main(value: int): int = {\n    return value!;\n}\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("unwrap requires an opt[...] or err[...] shell with a value type in V1")
        }),
        "Expected the unwrap non-shell diagnostic, got: {errors:?}"
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

#[test]
fn declaration_signature_lowering_keeps_alias_target_types_exact() {
    let typed = typecheck_fixture_folder(&[
        ("types.fol", "ali PathLabel: str\n"),
        ("main.fol", "var current: PathLabel = \"main\"\n"),
    ]);

    let (alias_id, alias) = find_typed_symbol(&typed, "PathLabel", SymbolKind::Alias);
    let (_current_id, current) = find_typed_symbol(&typed, "current", SymbolKind::ValueBinding);

    assert_eq!(
        typed.type_table().get(alias.declared_type.expect("alias should lower")),
        Some(&CheckedType::Builtin(BuiltinType::Str))
    );
    assert_eq!(
        typed.type_table().get(current.declared_type.expect("binding should lower")),
        Some(&CheckedType::Declared {
            symbol: alias_id,
            name: "PathLabel".to_string(),
            kind: DeclaredTypeKind::Alias,
        })
    );
}

#[test]
fn declaration_signature_lowering_records_entry_variant_payload_types() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "typ Token: ent = {\n\
             var Word: str = \"word\";\n\
             con Number: int = 1;\n\
         }\n",
    )]);

    let (_token_id, token) = find_typed_symbol(&typed, "Token", SymbolKind::Type);
    let CheckedType::Entry { variants } = typed
        .type_table()
        .get(token.declared_type.expect("entry type should lower"))
        .expect("entry type facts should exist")
    else {
        panic!("entry declaration should lower to an entry semantic type");
    };

    assert_eq!(variants.get("Word"), Some(&Some(typed.builtin_types().str_)));
    assert_eq!(variants.get("Number"), Some(&Some(typed.builtin_types().int)));
}

#[test]
fn declaration_signature_lowering_allows_forward_cross_file_alias_references() {
    let typed = typecheck_fixture_folder(&[
        ("00_main.fol", "var total: Count = 1\n"),
        ("10_types.fol", "ali Count: int\n"),
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
fn declaration_signature_lowering_allows_cross_file_named_type_references_in_routine_signatures() {
    let typed = typecheck_fixture_folder(&[
        (
            "00_main.fol",
            "fun[] load(item: model::User): model::User = {\n\
                 return item;\n\
             }\n",
        ),
        ("model/user.fol", "typ User: rec = {\n    name: str\n}\n"),
    ]);

    let (user_id, _user) = find_typed_symbol(&typed, "User", SymbolKind::Type);
    let (_load_id, load) = find_typed_symbol(&typed, "load", SymbolKind::Routine);
    let CheckedType::Routine(load_type) = typed
        .type_table()
        .get(load.declared_type.expect("routine should lower"))
        .expect("routine type should exist")
    else {
        panic!("routine declaration should lower to a routine semantic type");
    };

    assert_eq!(
        typed.type_table().get(load_type.params[0]),
        Some(&CheckedType::Declared {
            symbol: user_id,
            name: "User".to_string(),
            kind: DeclaredTypeKind::Type,
        })
    );
    assert_eq!(
        typed
            .type_table()
            .get(load_type.return_type.expect("routine return type should lower")),
        Some(&CheckedType::Declared {
            symbol: user_id,
            name: "User".to_string(),
            kind: DeclaredTypeKind::Type,
        })
    );
}

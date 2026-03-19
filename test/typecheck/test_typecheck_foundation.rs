use super::*;

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
        "fun[] demo(): int / str = {\n\
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
        "fun[] demo(): int / str = {\n\
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
        "fun[] demo(): int / str = {\n\
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
fn routine_error_calls_keep_recoverable_effects_on_call_references() {
    let typed = typecheck_fixture_folder(&[(
        "main.fol",
        "fun[] load(): int / str = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n\
         fun[] main(): bol = {\n\
             return check(load());\n\
         }\n",
    )]);

    let reference = find_typed_reference(&typed, "load", ReferenceKind::FunctionCall);

    assert_eq!(
        reference
            .resolved_type
            .and_then(|type_id| typed.type_table().get(type_id)),
        Some(&CheckedType::Builtin(BuiltinType::Int))
    );
    assert_eq!(
        reference
            .recoverable_effect
            .and_then(|effect| typed.type_table().get(effect.error_type)),
        Some(&CheckedType::Builtin(BuiltinType::Str))
    );
}

#[test]
fn inferred_bindings_reject_recoverable_call_results() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] load(): int / str = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n\
         fun[] main(): int = {\n\
             var current = load();\n\
             return 0;\n\
         }\n",
    )]);

    assert_eq!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error.message().contains("initializer for 'current'")
                && error
                    .message()
                    .contains("cannot use a routine result with '/ ErrorType' as a plain value")
        }),
        true,
        "Expected a strict binding diagnostic, got: {errors:?}"
    );
}

#[test]
fn plain_use_of_errorful_calls_rejects_plain_value_contexts() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] load(): int / str = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n\
         fun[] main(): int = {\n\
             var total: int = load() + 1;\n\
             return total;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("cannot use a routine result with '/ ErrorType' as a plain value")
        }),
        "Expected a plain-use errorful-call diagnostic, got: {errors:?}"
    );
}

#[test]
fn propagation_typing_rejects_matching_error_types_in_plain_value_contexts() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] load(): int / str = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n\
         fun[] main(): int / str = {\n\
             return load() + 1;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("cannot use a routine result with '/ ErrorType' as a plain value")
        }),
        "Expected a strict no-propagation diagnostic, got: {errors:?}"
    );
}

#[test]
fn propagation_typing_rejects_incompatible_error_types_in_plain_value_contexts() {
    let errors = typecheck_fixture_folder_errors(&[(
        "main.fol",
        "fun[] load(): int / str = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n\
         fun[] main(): int / int = {\n\
             return load() + 1;\n\
         }\n",
    )]);

    assert!(
        errors.iter().any(|error| {
            error.kind() == TypecheckErrorKind::InvalidInput
                && error
                    .message()
                    .contains("cannot use a routine result with '/ ErrorType' as a plain value")
        }),
        "Expected a strict no-propagation diagnostic, got: {errors:?}"
    );
}

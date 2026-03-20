use super::{
    lower_fixture_error, lower_fixture_workspace, lower_folder_fixture_error,
    lower_folder_fixture_workspace,
};
use crate::{LoweredInstrKind, LoweredOperand, LoweredTerminator, LoweringErrorKind};
use fol_parser::ast::AstParser;
use fol_resolver::{resolve_package_workspace, SymbolKind};
use fol_stream::FileStream;
use fol_typecheck::Typechecker;

#[test]
fn routine_body_lowering_keeps_local_initializers_and_final_expression_results() {
    let fixture = super::safe_temp_dir().join(format!(
        "fol_lower_body_exprs_{}.fol",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be monotonic enough for tmp names")
            .as_nanos()
    ));
    std::fs::write(
        &fixture,
        "fun[] main(): int = {\n    var value: int = 1\n    value\n}",
    )
    .expect("should write lowering body fixture");

    let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
        .expect("Should open lowering fixture");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Lowering fixture should parse");
    let resolved = resolve_package_workspace(syntax).expect("Lowering fixture should resolve");
    let typed = Typechecker::new()
        .check_resolved_workspace(resolved)
        .expect("Lowering fixture should typecheck");
    let lowered = crate::LoweringSession::new(typed)
        .lower_workspace()
        .expect("body lowering should succeed");

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .next()
        .expect("lowered routine should exist");
    let entry_block = routine
        .blocks
        .get(routine.entry_block)
        .expect("entry block should exist");

    assert_eq!(entry_block.instructions.len(), 3);
    assert_eq!(
        routine
            .instructions
            .get(crate::LoweredInstrId(0))
            .map(|instr| &instr.kind),
        Some(&LoweredInstrKind::Const(LoweredOperand::Int(1)))
    );
    assert!(
        matches!(
            routine
                .instructions
                .get(crate::LoweredInstrId(1))
                .map(|instr| &instr.kind),
            Some(LoweredInstrKind::StoreLocal { .. })
        ),
        "local binding initializer should lower into a store"
    );
    assert!(
        matches!(
            routine
                .instructions
                .get(crate::LoweredInstrId(2))
                .map(|instr| &instr.kind),
            Some(LoweredInstrKind::LoadLocal { .. })
        ),
        "final body expression should lower into a local load"
    );
    assert!(routine.body_result.is_some());
}

#[test]
fn assignment_lowering_emits_local_and_global_store_instructions() {
    let fixture = super::safe_temp_dir().join(format!(
        "fol_lower_assignment_exprs_{}.fol",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be monotonic enough for tmp names")
            .as_nanos()
    ));
    std::fs::write(
        &fixture,
        "var count: int = 0\nfun[] main(): int = {\n    var value: int = 1\n    value = 2\n    count = value\n    value\n}",
    )
    .expect("should write lowering assignment fixture");

    let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
        .expect("Should open lowering fixture");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Lowering fixture should parse");
    let resolved = resolve_package_workspace(syntax).expect("Lowering fixture should resolve");
    let typed = Typechecker::new()
        .check_resolved_workspace(resolved)
        .expect("Lowering fixture should typecheck");
    let lowered = crate::LoweringSession::new(typed)
        .lower_workspace()
        .expect("assignment lowering should succeed");

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .next()
        .expect("lowered routine should exist");

    assert!(
        routine
            .instructions
            .iter()
            .any(|instr| matches!(instr.kind, LoweredInstrKind::StoreLocal { .. })),
        "assignment to local bindings should lower into StoreLocal"
    );
    assert!(
        routine
            .instructions
            .iter()
            .any(|instr| matches!(instr.kind, LoweredInstrKind::StoreGlobal { .. })),
        "assignment to globals should lower into StoreGlobal"
    );
}

#[test]
fn call_lowering_emits_direct_callee_calls_for_plain_and_qualified_forms() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for tmp path")
        .as_nanos();
    let root = super::safe_temp_dir().join(format!("fol_lower_call_exprs_{stamp}"));
    let app_dir = root.join("app");
    let math_dir = app_dir.join("math");
    fs::create_dir_all(&math_dir).expect("should create nested namespace dir");
    fs::write(
        app_dir.join("main.fol"),
        "fun[] helper(): int = { 1 }\nfun[] main(): int = {\n    helper()\n    math::triple()\n}",
    )
    .expect("should write entry file");
    fs::write(math_dir.join("lib.fol"), "fun[exp] triple(): int = { 3 }\n")
        .expect("should write nested namespace file");

    let mut stream = FileStream::from_folder(app_dir.to_str().expect("utf8 temp path"))
        .expect("Should open lowering fixture");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Lowering fixture should parse");
    let resolved = resolve_package_workspace(syntax).expect("Lowering fixture should resolve");
    let typed = Typechecker::new()
        .check_resolved_workspace(resolved)
        .expect("Lowering fixture should typecheck");
    let lowered = crate::LoweringSession::new(typed)
        .lower_workspace()
        .expect("call lowering should succeed");

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");
    let call_instrs = routine
        .instructions
        .iter()
        .filter(|instr| matches!(instr.kind, LoweredInstrKind::Call { .. }))
        .collect::<Vec<_>>();

    assert_eq!(call_instrs.len(), 2);
}

#[test]
fn method_call_lowering_rewrites_receivers_into_direct_call_arguments() {
    let fixture = super::safe_temp_dir().join(format!(
        "fol_lower_method_exprs_{}.fol",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be monotonic enough for tmp names")
            .as_nanos()
    ));
    std::fs::write(
        &fixture,
        "fun (int)double(): int = { 2 }\nfun[] main(): int = {\n    var value: int = 1\n    value.double()\n}",
    )
    .expect("should write lowering method fixture");

    let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
        .expect("Should open lowering fixture");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Lowering fixture should parse");
    let resolved = resolve_package_workspace(syntax).expect("Lowering fixture should resolve");
    let typed = Typechecker::new()
        .check_resolved_workspace(resolved)
        .expect("Lowering fixture should typecheck");
    let lowered = crate::LoweringSession::new(typed)
        .lower_workspace()
        .expect("method call lowering should succeed");

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");
    let call = routine
        .instructions
        .iter()
        .find_map(|instr| match &instr.kind {
            LoweredInstrKind::Call { callee, args, .. } => Some((*callee, args.clone())),
            _ => None,
        })
        .expect("method body should contain a lowered call");

    assert_eq!(call.1.len(), 1);
}

#[test]
fn errorful_call_lowering_retains_explicit_error_type_metadata() {
    let lowered = lower_fixture_workspace(
        "fun[] load(): int / str = {\n\
             report \"bad\";\n\
             return 1;\n\
         }\n\
         fun[] main(): int / str = {\n\
             return load() || report \"forwarded\";\n\
         }\n",
    );

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");
    let call_error_type = routine
        .instructions
        .iter()
        .find_map(|instr| match &instr.kind {
            LoweredInstrKind::Call { error_type, .. } => *error_type,
            _ => None,
        })
        .expect("errorful call should retain an explicit lowered error type");

    assert_eq!(
        lowered.type_table().get(call_error_type),
        Some(&crate::LoweredType::Builtin(crate::LoweredBuiltinType::Str))
    );
    let signature = routine
        .signature
        .and_then(|signature| lowered.type_table().get(signature))
        .expect("main routine should retain a lowered signature");
    match signature {
        crate::LoweredType::Routine(signature) => {
            assert_eq!(
                signature
                    .error_type
                    .and_then(|error_type| lowered.type_table().get(error_type)),
                Some(&crate::LoweredType::Builtin(crate::LoweredBuiltinType::Str))
            );
        }
        other => panic!("expected lowered routine signature, got {other:?}"),
    }
}

#[test]
fn explicit_report_fallback_lowering_branches_and_reports_recoverable_calls() {
    let lowered = lower_fixture_workspace(concat!(
        "fun[] load(flag: bol): int / str = {\n",
        "    when(flag) {\n",
        "        case(true) { report \"bad\" }\n",
        "        * { return 7 }\n",
        "    }\n",
        "}\n",
        "fun[] main(flag: bol): int / str = {\n",
        "    return load(flag) || report \"forwarded\"\n",
        "}\n",
    ));

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");

    assert!(routine.instructions.iter().any(|instr| matches!(
        instr.kind,
        LoweredInstrKind::Call {
            error_type: Some(_),
            ..
        }
    )));
    assert!(routine
        .instructions
        .iter()
        .any(|instr| matches!(instr.kind, LoweredInstrKind::CheckRecoverable { .. })));
    assert!(!routine
        .instructions
        .iter()
        .any(|instr| matches!(instr.kind, LoweredInstrKind::ExtractRecoverableError { .. })));
    assert!(routine
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Some(LoweredTerminator::Branch { .. }))));
    assert!(routine
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Some(LoweredTerminator::Report { .. }))));
}

#[test]
fn check_lowering_observes_recoverable_bindings_without_propagation() {
    let lowered = lower_fixture_workspace(concat!(
        "fun[] load(flag: bol): int / str = {\n",
        "    when(flag) {\n",
        "        case(true) { report \"bad\" }\n",
        "        * { return 7 }\n",
        "    }\n",
        "}\n",
        "fun[] main(flag: bol): bol = {\n",
        "    return check(load(flag))\n",
        "}\n",
    ));

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");
    assert!(routine
        .instructions
        .iter()
        .any(|instr| matches!(instr.kind, LoweredInstrKind::CheckRecoverable { .. })));
    assert!(!routine
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Some(LoweredTerminator::Report { .. }))));
}

#[test]
fn pipe_or_default_lowering_branches_to_a_plain_fallback_value() {
    let lowered = lower_fixture_workspace(concat!(
        "fun[] load(flag: bol): int / str = {\n",
        "    when(flag) {\n",
        "        case(true) { report \"bad\" }\n",
        "        * { return 7 }\n",
        "    }\n",
        "}\n",
        "fun[] main(flag: bol): int = {\n",
        "    return load(flag) || 5\n",
        "}\n",
    ));

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");

    assert!(routine
        .instructions
        .iter()
        .any(|instr| matches!(instr.kind, LoweredInstrKind::CheckRecoverable { .. })));
    assert!(routine
        .instructions
        .iter()
        .any(|instr| matches!(instr.kind, LoweredInstrKind::UnwrapRecoverable { .. })));
    assert!(!routine
        .instructions
        .iter()
        .any(|instr| matches!(instr.kind, LoweredInstrKind::ExtractRecoverableError { .. })));
    assert!(routine
        .instructions
        .iter()
        .any(|instr| matches!(instr.kind, LoweredInstrKind::Const(LoweredOperand::Int(5)))));
    assert!(!routine.blocks.iter().any(|block| matches!(
        block.terminator,
        Some(LoweredTerminator::Report { .. } | LoweredTerminator::Panic { .. })
    )));
}

#[test]
fn pipe_or_report_lowering_uses_error_branch_reports() {
    let lowered = lower_fixture_workspace(concat!(
        "fun[] load(flag: bol): int / str = {\n",
        "    when(flag) {\n",
        "        case(true) { report \"bad\" }\n",
        "        * { return 7 }\n",
        "    }\n",
        "}\n",
        "fun[] main(flag: bol): int / str = {\n",
        "    return load(flag) || report \"fallback\"\n",
        "}\n",
    ));

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");

    assert!(routine
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Some(LoweredTerminator::Report { .. }))));
}

#[test]
fn pipe_or_panic_lowering_uses_error_branch_panics() {
    let lowered = lower_fixture_workspace(concat!(
        "fun[] load(flag: bol): int / str = {\n",
        "    when(flag) {\n",
        "        case(true) { report \"bad\" }\n",
        "        * { return 7 }\n",
        "    }\n",
        "}\n",
        "fun[] main(flag: bol): int = {\n",
        "    return load(flag) || panic \"fallback\"\n",
        "}\n",
    ));

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");

    assert!(routine
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Some(LoweredTerminator::Panic { .. }))));
}

#[test]
fn standalone_panic_lowering_uses_keyword_intrinsic_terminators() {
    let lowered = lower_fixture_workspace(concat!(
        "fun[] main(): int = {\n",
        "    panic \"boom\"\n",
        "}\n",
    ));

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");

    assert!(routine
        .blocks
        .iter()
        .any(|block| matches!(block.terminator, Some(LoweredTerminator::Panic { .. }))));
}

#[test]
fn field_access_lowering_emits_explicit_extraction_instructions() {
    let fixture = super::safe_temp_dir().join(format!(
        "fol_lower_field_exprs_{}.fol",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be monotonic enough for tmp names")
            .as_nanos()
    ));
    std::fs::write(
        &fixture,
        "typ Point: { x: int, y: int }\nfun[] main(point: Point): int = {\n    point.x\n}",
    )
    .expect("should write lowering field fixture");

    let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
        .expect("Should open lowering fixture");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Lowering fixture should parse");
    let resolved = resolve_package_workspace(syntax).expect("Lowering fixture should resolve");
    let typed = Typechecker::new()
        .check_resolved_workspace(resolved)
        .expect("Lowering fixture should typecheck");
    let lowered = crate::LoweringSession::new(typed)
        .lower_workspace()
        .expect("field access lowering should succeed");

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "main")
        .expect("main routine should exist");

    assert!(
        routine
            .instructions
            .iter()
            .any(|instr| matches!(instr.kind, LoweredInstrKind::FieldAccess { .. })),
        "record field access should lower into an explicit FieldAccess instruction"
    );
}

#[test]
fn index_access_lowering_emits_explicit_container_access_instructions() {
    let fixture = super::safe_temp_dir().join(format!(
        "fol_lower_index_exprs_{}.fol",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be monotonic enough for tmp names")
            .as_nanos()
    ));
    std::fs::write(
        &fixture,
        "fun[] head(values: vec[int]): int = {\n    values[0]\n}",
    )
    .expect("should write lowering index fixture");

    let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
        .expect("Should open lowering fixture");
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .expect("Lowering fixture should parse");
    let resolved = resolve_package_workspace(syntax).expect("Lowering fixture should resolve");
    let typed = Typechecker::new()
        .check_resolved_workspace(resolved)
        .expect("Lowering fixture should typecheck");
    let lowered = crate::LoweringSession::new(typed)
        .lower_workspace()
        .expect("index access lowering should succeed");

    let routine = lowered
        .entry_package()
        .routine_decls
        .values()
        .find(|routine| routine.name == "head")
        .expect("head routine should exist");

    assert!(
        routine
            .instructions
            .iter()
            .any(|instr| matches!(instr.kind, LoweredInstrKind::IndexAccess { .. })),
        "container index access should lower into an explicit IndexAccess instruction"
    );
}

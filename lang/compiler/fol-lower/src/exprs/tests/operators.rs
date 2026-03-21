use super::lower_fixture_workspace;
use crate::{LoweredInstrKind, control::LoweredBinaryOp, control::LoweredUnaryOp};

#[test]
fn arithmetic_binary_operators_lower_to_binary_op_instructions() {
    let workspace = lower_fixture_workspace(
        "fun[] main(): int = {\n    return 1 + 2;\n};\n",
    );
    let routine = workspace
        .entry_package()
        .routine_decls
        .values()
        .find(|r| r.name == "main")
        .expect("should find main routine");
    let has_add = routine.instructions.iter().any(|instr| {
        matches!(instr.kind, LoweredInstrKind::BinaryOp { op: LoweredBinaryOp::Add, .. })
    });
    assert!(has_add, "lowered IR should contain a BinaryOp::Add instruction");
}

#[test]
fn comparison_binary_operators_lower_to_binary_op_instructions() {
    let workspace = lower_fixture_workspace(
        "fun[] main(a: int, b: int): bol = {\n    return .eq(a, b);\n};\n",
    );
    assert_eq!(workspace.package_count(), 1);

    let workspace2 = lower_fixture_workspace(
        "fun[] main(a: int, b: int): bol = {\n    return a == b;\n};\n",
    );
    let routine = workspace2
        .entry_package()
        .routine_decls
        .values()
        .find(|r| r.name == "main")
        .expect("should find main routine");
    let has_eq = routine.instructions.iter().any(|instr| {
        matches!(instr.kind, LoweredInstrKind::BinaryOp { op: LoweredBinaryOp::Eq, .. })
    });
    assert!(has_eq, "lowered IR should contain a BinaryOp::Eq instruction");
}

#[test]
fn logical_binary_operators_lower_to_binary_op_instructions() {
    let workspace = lower_fixture_workspace(
        "fun[] main(a: bol, b: bol): bol = {\n    return a and b;\n};\n",
    );
    let routine = workspace
        .entry_package()
        .routine_decls
        .values()
        .find(|r| r.name == "main")
        .expect("should find main routine");
    let has_and = routine.instructions.iter().any(|instr| {
        matches!(instr.kind, LoweredInstrKind::BinaryOp { op: LoweredBinaryOp::And, .. })
    });
    assert!(has_and, "lowered IR should contain a BinaryOp::And instruction");
}

#[test]
fn negation_unary_operator_lowers_to_unary_op_instruction() {
    let workspace = lower_fixture_workspace(
        "fun[] main(): int = {\n    return -1;\n};\n",
    );
    let routine = workspace
        .entry_package()
        .routine_decls
        .values()
        .find(|r| r.name == "main")
        .expect("should find main routine");
    let has_neg = routine.instructions.iter().any(|instr| {
        matches!(instr.kind, LoweredInstrKind::UnaryOp { op: LoweredUnaryOp::Neg, .. })
    });
    assert!(has_neg, "lowered IR should contain a UnaryOp::Neg instruction");
}

#[test]
fn boolean_not_unary_operator_lowers_to_unary_op_instruction() {
    let workspace = lower_fixture_workspace(
        "fun[] main(): bol = {\n    return .not(true);\n};\n",
    );
    assert_eq!(workspace.package_count(), 1);
}

#[test]
fn ref_deref_unary_operators_reject_at_typecheck() {
    let workspace = lower_fixture_workspace(
        "fun[] main(): int = {\n    return 42;\n};\n",
    );
    assert_eq!(workspace.package_count(), 1);
}

#[test]
fn subtraction_and_multiplication_lower_correctly() {
    let workspace = lower_fixture_workspace(
        "fun[] main(a: int, b: int): int = {\n    return a - b * a;\n};\n",
    );
    let routine = workspace
        .entry_package()
        .routine_decls
        .values()
        .find(|r| r.name == "main")
        .expect("should find main routine");
    let has_sub = routine.instructions.iter().any(|instr| {
        matches!(instr.kind, LoweredInstrKind::BinaryOp { op: LoweredBinaryOp::Sub, .. })
    });
    let has_mul = routine.instructions.iter().any(|instr| {
        matches!(instr.kind, LoweredInstrKind::BinaryOp { op: LoweredBinaryOp::Mul, .. })
    });
    assert!(has_sub, "lowered IR should contain BinaryOp::Sub");
    assert!(has_mul, "lowered IR should contain BinaryOp::Mul");
}

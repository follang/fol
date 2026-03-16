use crate::{
    mangle_global_name, mangle_local_name, mangle_routine_name, BackendError, BackendErrorKind,
    BackendResult,
};
use fol_intrinsics::intrinsic_by_id;
use fol_lower::{LoweredInstr, LoweredInstrKind, LoweredOperand, LoweredRoutine};
use fol_resolver::PackageIdentity;

pub fn render_core_instruction(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    instruction: &LoweredInstr,
) -> BackendResult<String> {
    match &instruction.kind {
        LoweredInstrKind::Const(operand) => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            Ok(format!("let {result} = {};", render_operand(operand)))
        }
        LoweredInstrKind::LoadLocal { local } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let source = render_local_name(package_identity, routine, *local)?;
            Ok(format!("let {result} = {source}.clone();"))
        }
        LoweredInstrKind::StoreLocal { local, value } => {
            let target = render_local_name(package_identity, routine, *local)?;
            let value = render_local_name(package_identity, routine, *value)?;
            Ok(format!("{target} = {value}.clone();"))
        }
        LoweredInstrKind::LoadGlobal { global } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            Ok(format!(
                "let {result} = {}.get().expect(\"global initialized\").clone();",
                mangle_global_name(package_identity, *global, "global")
            ))
        }
        LoweredInstrKind::StoreGlobal { global, value } => {
            let value = render_local_name(package_identity, routine, *value)?;
            Ok(format!(
                "*{}.lock().expect(\"global lock\") = {value}.clone();",
                mangle_global_name(package_identity, *global, "global")
            ))
        }
        LoweredInstrKind::Call {
            callee,
            args,
            error_type: None,
        } => {
            let rendered_args = args
                .iter()
                .map(|local_id| render_local_name(package_identity, routine, *local_id))
                .collect::<BackendResult<Vec<_>>>()?
                .join(", ");
            let callee_name = mangle_routine_name(package_identity, *callee, "callee");
            match instruction.result {
                Some(_) => {
                    let result = rendered_result_local(package_identity, routine, instruction)?;
                    Ok(format!("let {result} = {callee_name}({rendered_args});"))
                }
                None => Ok(format!("{callee_name}({rendered_args});")),
            }
        }
        LoweredInstrKind::FieldAccess { base, field } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let base = render_local_name(package_identity, routine, *base)?;
            Ok(format!("let {result} = {base}.{field}.clone();"))
        }
        LoweredInstrKind::IntrinsicCall { intrinsic, args } => {
            let entry = intrinsic_by_id(*intrinsic).ok_or_else(|| {
                BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!("intrinsic id {:?} is not registered", intrinsic),
                )
            })?;
            let rendered_args = args
                .iter()
                .map(|local_id| render_local_name(package_identity, routine, *local_id))
                .collect::<BackendResult<Vec<_>>>()?;
            let expression = render_native_intrinsic_expression(entry.name, &rendered_args)?;
            match instruction.result {
                Some(_) => {
                    let result = rendered_result_local(package_identity, routine, instruction)?;
                    Ok(format!("let {result} = {expression};"))
                }
                None => Ok(format!("{expression};")),
            }
        }
        LoweredInstrKind::LengthOf { operand } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let operand = render_local_name(package_identity, routine, *operand)?;
            Ok(format!("let {result} = rt::len(&{operand});"))
        }
        LoweredInstrKind::RuntimeHook { intrinsic, args } => {
            let entry = intrinsic_by_id(*intrinsic).ok_or_else(|| {
                BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!("intrinsic id {:?} is not registered", intrinsic),
                )
            })?;
            match (entry.name, args.as_slice()) {
                ("echo", [value]) => {
                    let value = render_local_name(package_identity, routine, *value)?;
                    let rendered = format!("rt::echo({value})");
                    match instruction.result {
                        Some(_) => {
                            let result =
                                rendered_result_local(package_identity, routine, instruction)?;
                            Ok(format!("let {result} = {rendered};"))
                        }
                        None => Ok(format!("{rendered};")),
                    }
                }
                (other, _) => Err(BackendError::new(
                    BackendErrorKind::Unsupported,
                    format!("runtime hook emission is not implemented yet for '.{other}(...)'"),
                )),
            }
        }
        LoweredInstrKind::CheckRecoverable { operand } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let operand = render_local_name(package_identity, routine, *operand)?;
            Ok(format!("let {result} = rt::check_recoverable(&{operand});"))
        }
        LoweredInstrKind::UnwrapRecoverable { operand } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let operand = render_local_name(package_identity, routine, *operand)?;
            Ok(format!(
                "let {result} = {operand}.clone().into_value().expect(\"recoverable success\");"
            ))
        }
        LoweredInstrKind::ExtractRecoverableError { operand } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let operand = render_local_name(package_identity, routine, *operand)?;
            Ok(format!(
                "let {result} = {operand}.clone().into_error().expect(\"recoverable error\");"
            ))
        }
        other => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!("core instruction emission is not implemented yet for {other:?}"),
        )),
    }
}

fn render_native_intrinsic_expression(name: &str, args: &[String]) -> BackendResult<String> {
    match (name, args) {
        ("eq", [lhs, rhs]) => Ok(format!("{lhs} == {rhs}")),
        ("nq", [lhs, rhs]) => Ok(format!("{lhs} != {rhs}")),
        ("lt", [lhs, rhs]) => Ok(format!("{lhs} < {rhs}")),
        ("gt", [lhs, rhs]) => Ok(format!("{lhs} > {rhs}")),
        ("ge", [lhs, rhs]) => Ok(format!("{lhs} >= {rhs}")),
        ("le", [lhs, rhs]) => Ok(format!("{lhs} <= {rhs}")),
        ("not", [value]) => Ok(format!("!{value}")),
        (other, _) => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!("native Rust intrinsic emission is not implemented yet for '.{other}(...)'"),
        )),
    }
}

fn rendered_result_local(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    instruction: &LoweredInstr,
) -> BackendResult<String> {
    let Some(local_id) = instruction.result else {
        return Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            format!("instruction {:?} does not have a result local", instruction.id),
        ));
    };
    render_local_name(package_identity, routine, local_id)
}

fn render_local_name(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    local_id: fol_lower::LoweredLocalId,
) -> BackendResult<String> {
    let Some(local) = routine.locals.get(local_id) else {
        return Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            format!("lowered local {:?} is missing", local_id),
        ));
    };
    Ok(mangle_local_name(
        package_identity,
        routine.id,
        local_id,
        local.name.as_deref(),
    ))
}

fn render_operand(operand: &LoweredOperand) -> String {
    match operand {
        LoweredOperand::Local(_) => "/*local*/".to_string(),
        LoweredOperand::Global(_) => "/*global*/".to_string(),
        LoweredOperand::Int(value) => format!("{value}_i64"),
        LoweredOperand::Float(bits) => format!("f64::from_bits({bits})"),
        LoweredOperand::Bool(value) => value.to_string(),
        LoweredOperand::Char(value) => format!("{value:?}"),
        LoweredOperand::Str(value) => format!("rt::FolStr::from({value:?})"),
        LoweredOperand::Nil => "rt::FolOption::nil()".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::render_core_instruction;
    use crate::testing::package_identity;
    use fol_intrinsics::intrinsic_by_canonical_name;
    use fol_lower::{
        LoweredBlockId, LoweredBuiltinType, LoweredInstr, LoweredInstrId, LoweredInstrKind,
        LoweredLocal, LoweredLocalId, LoweredOperand, LoweredRoutine, LoweredRoutineId,
        LoweredTypeTable,
    };
    use fol_resolver::{PackageSourceKind, SourceUnitId, SymbolId};

    #[test]
    fn core_instruction_rendering_covers_constants_and_local_global_storage_shapes() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        let result_local = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let other_local = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("other".to_string()),
        });

        let const_instr = LoweredInstr {
            id: LoweredInstrId(0),
            result: Some(result_local),
            kind: LoweredInstrKind::Const(LoweredOperand::Int(7)),
        };
        let load_local = LoweredInstr {
            id: LoweredInstrId(1),
            result: Some(other_local),
            kind: LoweredInstrKind::LoadLocal { local: result_local },
        };
        let store_local = LoweredInstr {
            id: LoweredInstrId(2),
            result: None,
            kind: LoweredInstrKind::StoreLocal {
                local: result_local,
                value: other_local,
            },
        };

        let const_rendered =
            render_core_instruction(&package_identity, &routine, &const_instr).expect("const");
        let load_local_rendered =
            render_core_instruction(&package_identity, &routine, &load_local).expect("load");
        let store_local_rendered =
            render_core_instruction(&package_identity, &routine, &store_local).expect("store");

        assert!(const_rendered.contains("let l__pkg__entry__app__r0__l0__value = 7_i64;"));
        assert!(load_local_rendered.contains("let l__pkg__entry__app__r0__l1__other = l__pkg__entry__app__r0__l0__value.clone();"));
        assert!(store_local_rendered.contains("l__pkg__entry__app__r0__l0__value = l__pkg__entry__app__r0__l1__other.clone();"));

        let _ = SourceUnitId(0);
        let _ = SymbolId(0);
    }

    #[test]
    fn core_instruction_rendering_emits_plain_routine_calls_for_non_recoverable_sites() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(3), "main", LoweredBlockId(0));
        let arg_local = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let result_local = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("result".to_string()),
        });
        let call = LoweredInstr {
            id: LoweredInstrId(3),
            result: Some(result_local),
            kind: LoweredInstrKind::Call {
                callee: LoweredRoutineId(9),
                args: vec![arg_local],
                error_type: None,
            },
        };

        let rendered = render_core_instruction(&package_identity, &routine, &call).expect("call");

        assert!(rendered.contains("let l__pkg__entry__app__r3__l1__result = r__pkg__entry__app__r9__callee("));
        assert!(rendered.contains("l__pkg__entry__app__r3__l0__value"));
    }

    #[test]
    fn core_instruction_rendering_emits_record_field_accesses_as_native_member_reads() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(4), "main", LoweredBlockId(0));
        let base_local = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("user".to_string()),
        });
        let result_local = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("age".to_string()),
        });
        let access = LoweredInstr {
            id: LoweredInstrId(4),
            result: Some(result_local),
            kind: LoweredInstrKind::FieldAccess {
                base: base_local,
                field: "age".to_string(),
            },
        };

        let rendered =
            render_core_instruction(&package_identity, &routine, &access).expect("field access");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r4__l1__age = l__pkg__entry__app__r4__l0__user.age.clone();"
        );
    }

    #[test]
    fn core_instruction_rendering_emits_scalar_intrinsics_as_native_rust_ops() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(5), "main", LoweredBlockId(0));
        let lhs = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("lhs".to_string()),
        });
        let rhs = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("rhs".to_string()),
        });
        let bool_value = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });
        let eq_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("same".to_string()),
        });
        let not_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(4),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("flipped".to_string()),
        });
        let eq_instr = LoweredInstr {
            id: LoweredInstrId(5),
            result: Some(eq_result),
            kind: LoweredInstrKind::IntrinsicCall {
                intrinsic: intrinsic_by_canonical_name("eq").expect("eq").id,
                args: vec![lhs, rhs],
            },
        };
        let not_instr = LoweredInstr {
            id: LoweredInstrId(6),
            result: Some(not_result),
            kind: LoweredInstrKind::IntrinsicCall {
                intrinsic: intrinsic_by_canonical_name("not").expect("not").id,
                args: vec![bool_value],
            },
        };

        let eq_rendered =
            render_core_instruction(&package_identity, &routine, &eq_instr).expect("eq");
        let not_rendered =
            render_core_instruction(&package_identity, &routine, &not_instr).expect("not");

        assert_eq!(
            eq_rendered,
            "let l__pkg__entry__app__r5__l3__same = l__pkg__entry__app__r5__l0__lhs == l__pkg__entry__app__r5__l1__rhs;"
        );
        assert_eq!(
            not_rendered,
            "let l__pkg__entry__app__r5__l4__flipped = !l__pkg__entry__app__r5__l2__flag;"
        );
    }

    #[test]
    fn combined_core_instruction_snapshot_stays_stable() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(6), "main", LoweredBlockId(0));
        let lhs = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("lhs".to_string()),
        });
        let rhs = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("rhs".to_string()),
        });
        let flag = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });
        let tmp = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("tmp".to_string()),
        });
        let bool_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(4),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("same".to_string()),
        });

        let rendered = [
            LoweredInstr {
                id: LoweredInstrId(10),
                result: Some(tmp),
                kind: LoweredInstrKind::Const(LoweredOperand::Int(7)),
            },
            LoweredInstr {
                id: LoweredInstrId(11),
                result: Some(lhs),
                kind: LoweredInstrKind::LoadLocal { local: tmp },
            },
            LoweredInstr {
                id: LoweredInstrId(12),
                result: None,
                kind: LoweredInstrKind::StoreLocal {
                    local: rhs,
                    value: lhs,
                },
            },
            LoweredInstr {
                id: LoweredInstrId(13),
                result: Some(tmp),
                kind: LoweredInstrKind::Call {
                    callee: LoweredRoutineId(8),
                    args: vec![lhs, rhs],
                    error_type: None,
                },
            },
            LoweredInstr {
                id: LoweredInstrId(14),
                result: Some(bool_result),
                kind: LoweredInstrKind::IntrinsicCall {
                    intrinsic: intrinsic_by_canonical_name("eq").expect("eq").id,
                    args: vec![lhs, rhs],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(15),
                result: Some(bool_result),
                kind: LoweredInstrKind::IntrinsicCall {
                    intrinsic: intrinsic_by_canonical_name("not").expect("not").id,
                    args: vec![flag],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(16),
                result: Some(tmp),
                kind: LoweredInstrKind::FieldAccess {
                    base: rhs,
                    field: "count".to_string(),
                },
            },
        ]
        .iter()
        .map(|instruction| render_core_instruction(&package_identity, &routine, instruction))
        .collect::<Result<Vec<_>, _>>()
        .expect("snapshot should render")
        .join("\n");

        assert_eq!(
            rendered,
            concat!(
                "let l__pkg__entry__app__r6__l3__tmp = 7_i64;\n",
                "let l__pkg__entry__app__r6__l0__lhs = l__pkg__entry__app__r6__l3__tmp.clone();\n",
                "l__pkg__entry__app__r6__l1__rhs = l__pkg__entry__app__r6__l0__lhs.clone();\n",
                "let l__pkg__entry__app__r6__l3__tmp = r__pkg__entry__app__r8__callee(l__pkg__entry__app__r6__l0__lhs, l__pkg__entry__app__r6__l1__rhs);\n",
                "let l__pkg__entry__app__r6__l4__same = l__pkg__entry__app__r6__l0__lhs == l__pkg__entry__app__r6__l1__rhs;\n",
                "let l__pkg__entry__app__r6__l4__same = !l__pkg__entry__app__r6__l2__flag;\n",
                "let l__pkg__entry__app__r6__l3__tmp = l__pkg__entry__app__r6__l1__rhs.count.clone();"
            )
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_length_via_runtime_prelude() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(7), "main", LoweredBlockId(0));
        let source = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("items".to_string()),
        });
        let result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("count".to_string()),
        });
        let instruction = LoweredInstr {
            id: LoweredInstrId(20),
            result: Some(result),
            kind: LoweredInstrKind::LengthOf { operand: source },
        };

        let rendered =
            render_core_instruction(&package_identity, &routine, &instruction).expect("length");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r7__l1__count = rt::len(&l__pkg__entry__app__r7__l0__items);"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_echo_via_runtime_hook() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(8), "main", LoweredBlockId(0));
        let value = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("shown".to_string()),
        });
        let instruction = LoweredInstr {
            id: LoweredInstrId(21),
            result: Some(result),
            kind: LoweredInstrKind::RuntimeHook {
                intrinsic: intrinsic_by_canonical_name("echo").expect("echo").id,
                args: vec![value],
            },
        };

        let rendered =
            render_core_instruction(&package_identity, &routine, &instruction).expect("echo");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r8__l1__shown = rt::echo(l__pkg__entry__app__r8__l0__value);"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_check_recoverable_via_runtime_helper() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(9), "main", LoweredBlockId(0));
        let source = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: None,
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("failed".to_string()),
        });
        let instruction = LoweredInstr {
            id: LoweredInstrId(22),
            result: Some(result),
            kind: LoweredInstrKind::CheckRecoverable { operand: source },
        };

        let rendered =
            render_core_instruction(&package_identity, &routine, &instruction).expect("check");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r9__l1__failed = rt::check_recoverable(&l__pkg__entry__app__r9__l0__value);"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_unwrap_recoverable_success_lane() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(10), "main", LoweredBlockId(0));
        let source = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: None,
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("unwrapped".to_string()),
        });
        let instruction = LoweredInstr {
            id: LoweredInstrId(23),
            result: Some(result),
            kind: LoweredInstrKind::UnwrapRecoverable { operand: source },
        };

        let rendered =
            render_core_instruction(&package_identity, &routine, &instruction).expect("unwrap");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r10__l1__unwrapped = l__pkg__entry__app__r10__l0__value.clone().into_value().expect(\"recoverable success\");"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_recoverable_error_extraction() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut routine = LoweredRoutine::new(LoweredRoutineId(11), "main", LoweredBlockId(0));
        let source = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: None,
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: None,
            recoverable_error_type: None,
            name: Some("error".to_string()),
        });
        let instruction = LoweredInstr {
            id: LoweredInstrId(24),
            result: Some(result),
            kind: LoweredInstrKind::ExtractRecoverableError { operand: source },
        };

        let rendered =
            render_core_instruction(&package_identity, &routine, &instruction).expect("extract");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r11__l1__error = l__pkg__entry__app__r11__l0__value.clone().into_error().expect(\"recoverable error\");"
        );
    }
}

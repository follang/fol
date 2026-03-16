use crate::{
    mangle_global_name, mangle_local_name, mangle_routine_name, BackendError, BackendErrorKind,
    BackendResult,
};
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
        other => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!("core instruction emission is not implemented yet for {other:?}"),
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
}

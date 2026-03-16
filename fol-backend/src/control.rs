use crate::{mangle_local_name, BackendError, BackendErrorKind, BackendResult};
use fol_lower::{LoweredRoutine, LoweredTerminator};
use fol_resolver::PackageIdentity;

pub fn render_terminator(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    terminator: &LoweredTerminator,
) -> BackendResult<String> {
    match terminator {
        LoweredTerminator::Jump { target } => Ok(format!(
            "__fol_next_block = {}; continue;",
            target.0
        )),
        LoweredTerminator::Branch {
            condition,
            then_block,
            else_block,
        } => {
            let condition = render_local_name(package_identity, routine, *condition)?;
            Ok(format!(
                "if {condition} {{ __fol_next_block = {}; }} else {{ __fol_next_block = {}; }} continue;",
                then_block.0, else_block.0
            ))
        }
        LoweredTerminator::Return { value } => match value {
            Some(value) => Ok(format!(
                "return {};",
                render_local_name(package_identity, routine, *value)?
            )),
            None => Ok("return;".to_string()),
        },
        LoweredTerminator::Report { value } => match value {
            Some(value) => Ok(format!(
                "return rt::FolRecover::err({});",
                render_local_name(package_identity, routine, *value)?
            )),
            None => Ok("return rt::FolRecover::err(());".to_string()),
        },
        LoweredTerminator::Panic { value } => match value {
            Some(value) => Ok(format!(
                "panic!(\"{}\", rt::render_echo(&{}));",
                "{}",
                render_local_name(package_identity, routine, *value)?
            )),
            None => Ok("panic!(\"panic\");".to_string()),
        },
        LoweredTerminator::Unreachable => Ok("unreachable!();".to_string()),
    }
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

#[cfg(test)]
mod tests {
    use super::render_terminator;
    use crate::testing::package_identity;
    use fol_lower::{
        LoweredBlockId, LoweredBuiltinType, LoweredLocal, LoweredLocalId, LoweredRoutine,
        LoweredRoutineId, LoweredTerminator, LoweredTypeTable,
    };
    use fol_resolver::PackageSourceKind;

    #[test]
    fn terminator_rendering_emits_jump_shells() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let _table = LoweredTypeTable::new();
        let routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));

        let rendered = render_terminator(
            &package_identity,
            &routine,
            &LoweredTerminator::Jump {
                target: LoweredBlockId(7),
            },
        )
        .expect("jump");

        assert_eq!(rendered, "__fol_next_block = 7; continue;");
    }

    #[test]
    fn terminator_rendering_emits_branch_shells() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(1), "main", LoweredBlockId(0));
        let condition = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });

        let rendered = render_terminator(
            &package_identity,
            &routine,
            &LoweredTerminator::Branch {
                condition,
                then_block: LoweredBlockId(1),
                else_block: LoweredBlockId(2),
            },
        )
        .expect("branch");

        assert_eq!(
            rendered,
            "if l__pkg__entry__app__r1__l0__flag { __fol_next_block = 1; } else { __fol_next_block = 2; } continue;"
        );
    }

    #[test]
    fn terminator_rendering_emits_return_shells() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(2), "main", LoweredBlockId(0));
        let value = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });

        let with_value = render_terminator(
            &package_identity,
            &routine,
            &LoweredTerminator::Return { value: Some(value) },
        )
        .expect("return value");
        let empty = render_terminator(
            &package_identity,
            &routine,
            &LoweredTerminator::Return { value: None },
        )
        .expect("return empty");

        assert_eq!(with_value, "return l__pkg__entry__app__r2__l0__value;");
        assert_eq!(empty, "return;");
    }

    #[test]
    fn terminator_rendering_emits_report_and_panic_shells() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let str_id = table.intern_builtin(LoweredBuiltinType::Str);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(3), "main", LoweredBlockId(0));
        let value = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(str_id),
            recoverable_error_type: None,
            name: Some("message".to_string()),
        });

        let report = render_terminator(
            &package_identity,
            &routine,
            &LoweredTerminator::Report { value: Some(value) },
        )
        .expect("report");
        let panic = render_terminator(
            &package_identity,
            &routine,
            &LoweredTerminator::Panic { value: Some(value) },
        )
        .expect("panic");

        assert_eq!(
            report,
            "return rt::FolRecover::err(l__pkg__entry__app__r3__l0__message);"
        );
        assert_eq!(
            panic,
            "panic!(\"{}\", rt::render_echo(&l__pkg__entry__app__r3__l0__message));"
        );
    }

    #[test]
    fn terminator_rendering_emits_unreachable_shells() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let routine = LoweredRoutine::new(LoweredRoutineId(4), "main", LoweredBlockId(0));

        let rendered =
            render_terminator(&package_identity, &routine, &LoweredTerminator::Unreachable)
                .expect("unreachable");

        assert_eq!(rendered, "unreachable!();");
    }

    #[test]
    fn control_flow_snapshot_stays_stable() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let str_id = table.intern_builtin(LoweredBuiltinType::Str);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(5), "main", LoweredBlockId(0));
        let flag = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });
        let value = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let message = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(str_id),
            recoverable_error_type: None,
            name: Some("message".to_string()),
        });

        let rendered = [
            LoweredTerminator::Jump {
                target: LoweredBlockId(3),
            },
            LoweredTerminator::Branch {
                condition: flag,
                then_block: LoweredBlockId(1),
                else_block: LoweredBlockId(2),
            },
            LoweredTerminator::Return { value: Some(value) },
            LoweredTerminator::Report {
                value: Some(message),
            },
            LoweredTerminator::Panic {
                value: Some(message),
            },
            LoweredTerminator::Unreachable,
        ]
        .iter()
        .map(|terminator| render_terminator(&package_identity, &routine, terminator))
        .collect::<Result<Vec<_>, _>>()
        .expect("control snapshot renders")
        .join("\n");

        assert_eq!(
            rendered,
            concat!(
                "__fol_next_block = 3; continue;\n",
                "if l__pkg__entry__app__r5__l0__flag { __fol_next_block = 1; } else { __fol_next_block = 2; } continue;\n",
                "return l__pkg__entry__app__r5__l1__value;\n",
                "return rt::FolRecover::err(l__pkg__entry__app__r5__l2__message);\n",
                "panic!(\"{}\", rt::render_echo(&l__pkg__entry__app__r5__l2__message));\n",
                "unreachable!();"
            )
        );
    }
}

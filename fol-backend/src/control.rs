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
        other => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!("terminator emission is not implemented yet for {other:?}"),
        )),
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
}

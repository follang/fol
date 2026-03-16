use crate::{
    mangle_global_name, mangle_local_name, mangle_routine_name, render_rust_type, BackendError,
    BackendErrorKind, BackendResult,
};
use fol_lower::{
    LoweredGlobal, LoweredRoutine, LoweredRoutineType, LoweredType, LoweredTypeId, LoweredTypeTable,
};
use fol_resolver::PackageIdentity;

pub fn render_global_declaration(
    package_identity: &PackageIdentity,
    global: &LoweredGlobal,
    type_table: &LoweredTypeTable,
) -> BackendResult<String> {
    let name = mangle_global_name(package_identity, global.id, &global.name);
    let value_type = render_rust_type(type_table, global.type_id)?;

    Ok(if global.mutable {
        format!(
            "pub static {name}: std::sync::LazyLock<std::sync::Mutex<{value_type}>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(todo!()));\n"
        )
    } else {
        format!(
            "pub static {name}: std::sync::LazyLock<{value_type}> = std::sync::LazyLock::new(|| todo!());\n"
        )
    })
}

pub fn render_routine_signature(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    type_table: &LoweredTypeTable,
) -> BackendResult<String> {
    let signature = routine_signature(type_table, routine.signature)?;

    let mut params = Vec::new();
    if let Some(receiver_type) = routine.receiver_type {
        params.push(format!("receiver: {}", render_rust_type(type_table, receiver_type)?));
    }
    params.extend(render_param_list(package_identity, routine, signature, type_table)?);

    let return_type = render_routine_return_type(signature, type_table)?;

    Ok(format!(
        "pub fn {}({}){}",
        mangle_routine_name(package_identity, routine.id, &routine.name),
        params.join(", "),
        return_type
    ))
}

fn routine_signature<'a>(
    type_table: &'a LoweredTypeTable,
    signature_id: Option<LoweredTypeId>,
) -> BackendResult<&'a LoweredRoutineType> {
    let Some(signature_id) = signature_id else {
        return Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            "routine is missing a lowered signature",
        ));
    };
    match type_table.get(signature_id) {
        Some(LoweredType::Routine(signature)) => Ok(signature),
        Some(other) => Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            format!("routine signature id did not point to a routine type: {other:?}"),
        )),
        None => Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            format!("routine signature type {:?} is missing", signature_id),
        )),
    }
}

fn render_param_list(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    signature: &LoweredRoutineType,
    type_table: &LoweredTypeTable,
) -> BackendResult<Vec<String>> {
    routine
        .params
        .iter()
        .enumerate()
        .map(|(index, local_id)| {
            let local = routine.locals.get(*local_id).ok_or_else(|| {
                BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!("routine parameter local {:?} is missing", local_id),
                )
            })?;
            let type_id = signature.params.get(index).copied().ok_or_else(|| {
                BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!("routine parameter {} is missing a signature type", index),
                )
            })?;
            Ok(format!(
                "{}: {}",
                mangle_local_name(package_identity, routine.id, *local_id, local.name.as_deref()),
                render_rust_type(type_table, type_id)?
            ))
        })
        .collect()
}

fn render_routine_return_type(
    signature: &LoweredRoutineType,
    type_table: &LoweredTypeTable,
) -> BackendResult<String> {
    let success_type = match signature.return_type {
        Some(return_type) => render_rust_type(type_table, return_type)?,
        None => "()".to_string(),
    };
    Ok(match signature.error_type {
        Some(error_type) => format!(
            " -> rt::FolRecover<{}, {}>",
            success_type,
            render_rust_type(type_table, error_type)?
        ),
        None if signature.return_type.is_some() => format!(" -> {success_type}"),
        None => String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{render_global_declaration, render_routine_signature};
    use crate::testing::package_identity;
    use fol_lower::{
        LoweredBlockId, LoweredBuiltinType, LoweredGlobal, LoweredGlobalId, LoweredLocal,
        LoweredLocalId, LoweredRoutine, LoweredRoutineId, LoweredRoutineType, LoweredType,
        LoweredTypeTable,
    };
    use fol_resolver::{PackageSourceKind, SourceUnitId, SymbolId};

    #[test]
    fn global_declaration_rendering_emits_lazy_shells_for_mutable_and_immutable_globals() {
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let immutable = LoweredGlobal {
            id: LoweredGlobalId(0),
            symbol_id: SymbolId(20),
            source_unit_id: SourceUnitId(0),
            name: "answer".to_string(),
            type_id: int_id,
            recoverable_error_type: None,
            mutable: false,
        };
        let mutable = LoweredGlobal {
            id: LoweredGlobalId(1),
            symbol_id: SymbolId(21),
            source_unit_id: SourceUnitId(0),
            name: "counter".to_string(),
            type_id: int_id,
            recoverable_error_type: None,
            mutable: true,
        };

        let immutable_rendered =
            render_global_declaration(&package_identity, &immutable, &table).expect("global");
        let mutable_rendered =
            render_global_declaration(&package_identity, &mutable, &table).expect("global");

        assert!(immutable_rendered.contains("pub static g__pkg__entry__app__g0__answer"));
        assert!(immutable_rendered.contains("std::sync::LazyLock<rt::FolInt>"));
        assert!(mutable_rendered.contains("pub static g__pkg__entry__app__g1__counter"));
        assert!(mutable_rendered.contains("std::sync::Mutex<rt::FolInt>"));
    }

    #[test]
    fn routine_signature_rendering_covers_plain_and_receiver_qualified_routines() {
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let signature_id = table.intern(LoweredType::Routine(LoweredRoutineType {
            params: vec![bool_id],
            return_type: Some(int_id),
            error_type: None,
        }));
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut plain = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        plain.signature = Some(signature_id);
        let local_id = plain.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });
        plain.params.push(local_id);

        let mut method = LoweredRoutine::new(LoweredRoutineId(1), "tick", LoweredBlockId(0));
        method.signature = Some(signature_id);
        method.receiver_type = Some(int_id);
        let method_param = method.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });
        method.params.push(method_param);

        let plain_rendered =
            render_routine_signature(&package_identity, &plain, &table).expect("signature");
        let method_rendered =
            render_routine_signature(&package_identity, &method, &table).expect("signature");

        assert!(plain_rendered.contains("pub fn r__pkg__entry__app__r0__main("));
        assert!(plain_rendered.contains("l__pkg__entry__app__r0__l0__flag: rt::FolBool"));
        assert!(plain_rendered.ends_with(" -> rt::FolInt"));
        assert!(method_rendered.contains("receiver: rt::FolInt"));
        assert!(method_rendered.contains("l__pkg__entry__app__r1__l0__flag: rt::FolBool"));
    }

    #[test]
    fn routine_signature_rendering_wraps_recoverable_returns_in_runtime_abi() {
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let str_id = table.intern_builtin(LoweredBuiltinType::Str);
        let signature_id = table.intern(LoweredType::Routine(LoweredRoutineType {
            params: vec![],
            return_type: Some(int_id),
            error_type: Some(str_id),
        }));
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut routine = LoweredRoutine::new(LoweredRoutineId(2), "load", LoweredBlockId(0));
        routine.signature = Some(signature_id);

        let rendered =
            render_routine_signature(&package_identity, &routine, &table).expect("signature");

        assert!(rendered.contains("pub fn r__pkg__entry__app__r2__load()"));
        assert!(rendered.ends_with(" -> rt::FolRecover<rt::FolInt, rt::FolStr>"));
    }
}

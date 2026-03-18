use crate::{
    mangle_global_name, mangle_local_name, mangle_routine_name, mangle_type_name, BackendError,
    BackendErrorKind, BackendResult,
};
use fol_intrinsics::intrinsic_by_id;
use fol_lower::{
    control::LoweredLinearKind, LoweredGlobal, LoweredInstr, LoweredInstrKind, LoweredOperand,
    LoweredRoutine, LoweredType, LoweredTypeDecl, LoweredTypeTable, LoweredWorkspace,
};
use fol_resolver::PackageIdentity;

pub fn render_core_instruction(
    package_identity: &PackageIdentity,
    type_table: &LoweredTypeTable,
    routine: &LoweredRoutine,
    instruction: &LoweredInstr,
) -> BackendResult<String> {
    render_core_instruction_in_workspace(None, package_identity, type_table, routine, instruction)
}

pub fn render_core_instruction_in_workspace(
    workspace: Option<&LoweredWorkspace>,
    package_identity: &PackageIdentity,
    type_table: &LoweredTypeTable,
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
            let (global_identity, global_decl) = resolve_global_decl(workspace, *global)?;
            Ok(format!(
                "let {result} = {};",
                render_global_load(workspace, global_identity, global_decl)?
            ))
        }
        LoweredInstrKind::StoreGlobal { global, value } => {
            let (global_identity, global_decl) = resolve_global_decl(workspace, *global)?;
            let value = render_local_name(package_identity, routine, *value)?;
            if !global_decl.mutable {
                return Err(BackendError::new(
                    BackendErrorKind::Unsupported,
                    format!(
                        "store emission is not implemented for immutable global '{}'",
                        global_decl.name
                    ),
                ));
            }
            let global_path = format!(
                "{}::{}",
                render_namespace_module_path(
                    workspace,
                    global_identity,
                    global_decl.source_unit_id
                )?,
                mangle_global_name(global_identity, *global, &global_decl.name)
            );
            Ok(format!(
                "*{global_path}.get_or_init(|| std::sync::Mutex::new(Default::default())).lock().expect(\"global lock\") = {value}.clone();",
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
            let (callee_identity, callee_decl) = resolve_routine_decl(workspace, *callee)?;
            let callee_name = render_routine_path(workspace, callee_identity, callee_decl)?;
            match instruction.result {
                Some(_) => {
                    let result = rendered_result_local(package_identity, routine, instruction)?;
                    Ok(format!("let {result} = {callee_name}({rendered_args});"))
                }
                None => Ok(format!("{callee_name}({rendered_args});")),
            }
        }
        LoweredInstrKind::Call {
            callee,
            args,
            error_type: Some(_),
        } => {
            let rendered_args = args
                .iter()
                .map(|local_id| render_local_name(package_identity, routine, *local_id))
                .collect::<BackendResult<Vec<_>>>()?
                .join(", ");
            let (callee_identity, callee_decl) = resolve_routine_decl(workspace, *callee)?;
            let callee_name = render_routine_path(workspace, callee_identity, callee_decl)?;
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
        LoweredInstrKind::ConstructOptional { value, .. } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let expression = match value {
                Some(value) => {
                    let value = render_local_name(package_identity, routine, *value)?;
                    format!("rt::FolOption::some({value}.clone())")
                }
                None => "rt::FolOption::nil()".to_string(),
            };
            Ok(format!("let {result} = {expression};"))
        }
        LoweredInstrKind::ConstructError { value, .. } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let expression = match value {
                Some(value) => {
                    let value = render_local_name(package_identity, routine, *value)?;
                    format!("rt::FolError::new({value}.clone())")
                }
                None => "rt::FolError::new(())".to_string(),
            };
            Ok(format!("let {result} = {expression};"))
        }
        LoweredInstrKind::UnwrapShell { operand } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let operand_name = render_local_name(package_identity, routine, *operand)?;
            let operand_local = routine.locals.get(*operand).ok_or_else(|| {
                BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!("lowered local {:?} is missing", operand),
                )
            })?;
            let Some(type_id) = operand_local.type_id else {
                return Err(BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!(
                        "shell operand local {:?} does not have a lowered type",
                        operand
                    ),
                ));
            };
            let expression = match type_table.get(type_id) {
                Some(LoweredType::Optional { .. }) => format!(
                    "rt::unwrap_optional_shell({operand_name}.clone()).expect(\"optional shell\")"
                ),
                Some(LoweredType::Error { .. }) => {
                    format!("rt::unwrap_error_shell({operand_name}.clone())")
                }
                other => return Err(BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!(
                        "shell unwrap emission expected optional/error local but found {other:?}"
                    ),
                )),
            };
            Ok(format!("let {result} = {expression};"))
        }
        LoweredInstrKind::ConstructLinear { kind, elements, .. } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let elements = render_local_list(package_identity, routine, elements)?;
            let expression = match kind {
                LoweredLinearKind::Array => format!("[{elements}]"),
                LoweredLinearKind::Vector => format!("rt::FolVec::from_items(vec![{elements}])"),
                LoweredLinearKind::Sequence => format!("rt::FolSeq::from_items(vec![{elements}])"),
            };
            Ok(format!("let {result} = {expression};"))
        }
        LoweredInstrKind::ConstructSet { members, .. } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let members = render_local_list(package_identity, routine, members)?;
            Ok(format!(
                "let {result} = rt::FolSet::from_items(vec![{members}]);"
            ))
        }
        LoweredInstrKind::ConstructMap { entries, .. } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let entries = entries
                .iter()
                .map(|(key, value)| {
                    Ok(format!(
                        "({}, {})",
                        render_clone_expr(package_identity, routine, *key)?,
                        render_clone_expr(package_identity, routine, *value)?
                    ))
                })
                .collect::<BackendResult<Vec<_>>>()?
                .join(", ");
            Ok(format!(
                "let {result} = rt::FolMap::from_pairs(vec![{entries}]);"
            ))
        }
        LoweredInstrKind::IndexAccess { container, index } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let container_name = render_local_name(package_identity, routine, *container)?;
            let index_name = render_local_name(package_identity, routine, *index)?;
            let container_local = routine.locals.get(*container).ok_or_else(|| {
                BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!("lowered local {:?} is missing", container),
                )
            })?;
            let Some(type_id) = container_local.type_id else {
                return Err(BackendError::new(
                    BackendErrorKind::InvalidInput,
                    format!(
                        "index container local {:?} does not have a lowered type",
                        container
                    ),
                ));
            };
            let expression = match type_table.get(type_id) {
                Some(LoweredType::Array { .. }) => format!(
                    "rt::index_array(&{container_name}, {index_name}.clone()).expect(\"array index\").clone()"
                ),
                Some(LoweredType::Vector { .. }) => format!(
                    "rt::index_vec(&{container_name}, {index_name}.clone()).expect(\"vector index\").clone()"
                ),
                Some(LoweredType::Sequence { .. }) => format!(
                    "rt::index_seq(&{container_name}, {index_name}.clone()).expect(\"sequence index\").clone()"
                ),
                Some(LoweredType::Map { .. }) => format!(
                    "rt::lookup_map(&{container_name}, &{index_name}).expect(\"map key\").clone()"
                ),
                other => {
                    return Err(BackendError::new(
                        BackendErrorKind::InvalidInput,
                        format!("index emission expected array/vector/sequence/map local but found {other:?}"),
                    ))
                }
            };
            Ok(format!("let {result} = {expression};"))
        }
        LoweredInstrKind::ConstructRecord { type_id, fields } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let (type_identity, type_decl) = resolve_type_decl(workspace, *type_id)?;
            let type_name = render_type_path(workspace, type_identity, type_decl)?;
            let rendered_fields = fields
                .iter()
                .map(|(field, local)| {
                    Ok(format!(
                        "{field}: {}",
                        render_clone_expr(package_identity, routine, *local)?
                    ))
                })
                .collect::<BackendResult<Vec<_>>>()?
                .join(", ");
            Ok(format!(
                "let {result} = {type_name} {{ {rendered_fields} }};"
            ))
        }
        LoweredInstrKind::ConstructEntry {
            type_id,
            variant,
            payload,
        } => {
            let result = rendered_result_local(package_identity, routine, instruction)?;
            let (type_identity, type_decl) = resolve_type_decl(workspace, *type_id)?;
            let type_name = render_type_path(workspace, type_identity, type_decl)?;
            let expression = match payload {
                Some(payload) => format!(
                    "{type_name}::{variant}({})",
                    render_clone_expr(package_identity, routine, *payload)?
                ),
                None => format!("{type_name}::{variant}"),
            };
            Ok(format!("let {result} = {expression};"))
        }
        other => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!("core instruction emission is not implemented yet for {other:?}"),
        )),
    }
}

fn resolve_global_decl(
    workspace: Option<&LoweredWorkspace>,
    global_id: fol_lower::LoweredGlobalId,
) -> BackendResult<(&PackageIdentity, &LoweredGlobal)> {
    let Some(workspace) = workspace else {
        return Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!(
                "workspace-aware global emission is required for global {:?}",
                global_id
            ),
        ));
    };
    workspace
        .packages()
        .find_map(|package| {
            package
                .global_decls
                .get(&global_id)
                .map(|global| (&package.identity, global))
        })
        .ok_or_else(|| {
            BackendError::new(
                BackendErrorKind::InvalidInput,
                format!(
                    "lowered global {:?} is missing from the workspace",
                    global_id
                ),
            )
        })
}

fn resolve_routine_decl(
    workspace: Option<&LoweredWorkspace>,
    routine_id: fol_lower::LoweredRoutineId,
) -> BackendResult<(&PackageIdentity, &LoweredRoutine)> {
    let Some(workspace) = workspace else {
        return Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!(
                "workspace-aware routine emission is required for routine {:?}",
                routine_id
            ),
        ));
    };
    workspace
        .packages()
        .find_map(|package| {
            package
                .routine_decls
                .get(&routine_id)
                .map(|routine| (&package.identity, routine))
        })
        .ok_or_else(|| {
            BackendError::new(
                BackendErrorKind::InvalidInput,
                format!(
                    "lowered routine {:?} is missing from the workspace",
                    routine_id
                ),
            )
        })
}

fn resolve_type_decl(
    workspace: Option<&LoweredWorkspace>,
    runtime_type: fol_lower::LoweredTypeId,
) -> BackendResult<(&PackageIdentity, &LoweredTypeDecl)> {
    let Some(workspace) = workspace else {
        return Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!(
                "workspace-aware aggregate emission is required for type {:?}",
                runtime_type
            ),
        ));
    };
    workspace
        .packages()
        .find_map(|package| {
            package
                .type_decls
                .values()
                .find(|type_decl| type_decl.runtime_type == runtime_type)
                .map(|type_decl| (&package.identity, type_decl))
        })
        .ok_or_else(|| {
            BackendError::new(
                BackendErrorKind::InvalidInput,
                format!(
                    "lowered type {:?} does not have a rendered declaration owner",
                    runtime_type
                ),
            )
        })
}

fn render_global_load(
    workspace: Option<&LoweredWorkspace>,
    global_identity: &PackageIdentity,
    global: &LoweredGlobal,
) -> BackendResult<String> {
    let global_name = format!(
        "{}::{}",
        render_namespace_module_path(workspace, global_identity, global.source_unit_id)?,
        mangle_global_name(global_identity, global.id, &global.name)
    );
    if global.mutable {
        Ok(format!(
            "{}.get_or_init(|| std::sync::Mutex::new(Default::default())).lock().expect(\"global lock\").clone()",
            global_name
        ))
    } else {
        Ok(format!(
            "{global_name}.get_or_init(Default::default).clone()"
        ))
    }
}

fn render_routine_path(
    workspace: Option<&LoweredWorkspace>,
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
) -> BackendResult<String> {
    Ok(format!(
        "{}::{}",
        render_namespace_module_path(
            workspace,
            package_identity,
            routine.source_unit_id.ok_or_else(|| BackendError::new(
                BackendErrorKind::InvalidInput,
                format!("routine '{}' is missing a source unit", routine.name),
            ))?,
        )?,
        mangle_routine_name(package_identity, routine.id, &routine.name)
    ))
}

fn render_type_path(
    workspace: Option<&LoweredWorkspace>,
    package_identity: &PackageIdentity,
    type_decl: &LoweredTypeDecl,
) -> BackendResult<String> {
    Ok(format!(
        "{}::{}",
        render_namespace_module_path(workspace, package_identity, type_decl.source_unit_id)?,
        mangle_type_name(package_identity, type_decl.runtime_type, &type_decl.name)
    ))
}

fn render_namespace_module_path(
    workspace: Option<&LoweredWorkspace>,
    package_identity: &PackageIdentity,
    source_unit_id: fol_resolver::SourceUnitId,
) -> BackendResult<String> {
    let Some(workspace) = workspace else {
        return Err(BackendError::new(
            BackendErrorKind::Unsupported,
            "workspace-aware namespace emission is required",
        ));
    };
    let package = workspace.package(package_identity).ok_or_else(|| {
        BackendError::new(
            BackendErrorKind::InvalidInput,
            format!(
                "package '{}' is missing from workspace",
                package_identity.display_name
            ),
        )
    })?;
    let source_unit = package
        .source_units
        .iter()
        .find(|source_unit| source_unit.source_unit_id == source_unit_id)
        .ok_or_else(|| {
            BackendError::new(
                BackendErrorKind::InvalidInput,
                format!(
                    "source unit {:?} is missing from package '{}'",
                    source_unit_id, package_identity.display_name
                ),
            )
        })?;
    let mut segments = source_unit
        .namespace
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(crate::sanitize_backend_ident)
        .collect::<Vec<_>>();
    if segments.first().is_some_and(|segment| {
        segment == &crate::sanitize_backend_ident(&package_identity.display_name)
    }) {
        segments.remove(0);
    }
    let namespace_segment = match segments.as_slice() {
        [] => "root".to_string(),
        parts => parts.join("::"),
    };
    Ok(format!(
        "crate::packages::{}::{}",
        crate::mangle_package_module_name(package_identity),
        namespace_segment
    ))
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

fn render_local_list(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    locals: &[fol_lower::LoweredLocalId],
) -> BackendResult<String> {
    locals
        .iter()
        .map(|local| render_clone_expr(package_identity, routine, *local))
        .collect::<BackendResult<Vec<_>>>()
        .map(|items| items.join(", "))
}

fn render_clone_expr(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    local_id: fol_lower::LoweredLocalId,
) -> BackendResult<String> {
    let name = render_local_name(package_identity, routine, local_id)?;
    Ok(format!("{name}.clone()"))
}

fn rendered_result_local(
    package_identity: &PackageIdentity,
    routine: &LoweredRoutine,
    instruction: &LoweredInstr,
) -> BackendResult<String> {
    let Some(local_id) = instruction.result else {
        return Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            format!(
                "instruction {:?} does not have a result local",
                instruction.id
            ),
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
        LoweredOperand::Local(_) => {
            r#"compile_error!("unimplemented operand: Local")"#.to_string()
        }
        LoweredOperand::Global(_) => {
            r#"compile_error!("unimplemented operand: Global")"#.to_string()
        }
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
        LoweredBlockId, LoweredBuiltinType, LoweredFieldLayout, LoweredInstr, LoweredInstrId,
        LoweredInstrKind, LoweredLocal, LoweredLocalId, LoweredOperand, LoweredPackage,
        LoweredRecoverableAbi, LoweredRoutine, LoweredRoutineId, LoweredSourceMap, LoweredType,
        LoweredTypeDecl, LoweredTypeDeclKind, LoweredTypeTable, LoweredVariantLayout,
        LoweredWorkspace,
    };
    use fol_resolver::{PackageSourceKind, SourceUnitId, SymbolId};
    use std::collections::BTreeMap;

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
            kind: LoweredInstrKind::LoadLocal {
                local: result_local,
            },
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
            render_core_instruction(&package_identity, &table, &routine, &const_instr)
                .expect("const");
        let load_local_rendered =
            render_core_instruction(&package_identity, &table, &routine, &load_local)
                .expect("load");
        let store_local_rendered =
            render_core_instruction(&package_identity, &table, &routine, &store_local)
                .expect("store");

        assert!(const_rendered.contains("let l__pkg__entry__app__r0__l0__value = 7_i64;"));
        assert!(load_local_rendered.contains(
            "let l__pkg__entry__app__r0__l1__other = l__pkg__entry__app__r0__l0__value.clone();"
        ));
        assert!(store_local_rendered.contains(
            "l__pkg__entry__app__r0__l0__value = l__pkg__entry__app__r0__l1__other.clone();"
        ));

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

        let rendered =
            render_core_instruction(&package_identity, &table, &routine, &call).expect("call");

        assert!(rendered
            .contains("let l__pkg__entry__app__r3__l1__result = r__pkg__entry__app__r9__callee("));
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

        let rendered = render_core_instruction(&package_identity, &table, &routine, &access)
            .expect("field access");

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
            render_core_instruction(&package_identity, &table, &routine, &eq_instr).expect("eq");
        let not_rendered =
            render_core_instruction(&package_identity, &table, &routine, &not_instr).expect("not");

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
        .map(|instruction| {
            render_core_instruction(&package_identity, &table, &routine, instruction)
        })
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

        let rendered = render_core_instruction(&package_identity, &table, &routine, &instruction)
            .expect("length");

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

        let rendered = render_core_instruction(&package_identity, &table, &routine, &instruction)
            .expect("echo");

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

        let rendered = render_core_instruction(&package_identity, &table, &routine, &instruction)
            .expect("check");

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

        let rendered = render_core_instruction(&package_identity, &table, &routine, &instruction)
            .expect("unwrap");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r10__l1__unwrapped = l__pkg__entry__app__r10__l0__value.clone().into_value().expect(\"recoverable success\");"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_recoverable_error_extraction() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let table = LoweredTypeTable::new();
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

        let rendered = render_core_instruction(&package_identity, &table, &routine, &instruction)
            .expect("extract");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r11__l1__error = l__pkg__entry__app__r11__l0__value.clone().into_error().expect(\"recoverable error\");"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_optional_shell_construction() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let optional_id = table.intern(fol_lower::LoweredType::Optional { inner: int_id });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(12), "main", LoweredBlockId(0));
        let payload = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let some_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(optional_id),
            recoverable_error_type: None,
            name: Some("maybe".to_string()),
        });
        let nil_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(optional_id),
            recoverable_error_type: None,
            name: Some("empty".to_string()),
        });
        let some_instr = LoweredInstr {
            id: LoweredInstrId(25),
            result: Some(some_result),
            kind: LoweredInstrKind::ConstructOptional {
                type_id: optional_id,
                value: Some(payload),
            },
        };
        let nil_instr = LoweredInstr {
            id: LoweredInstrId(26),
            result: Some(nil_result),
            kind: LoweredInstrKind::ConstructOptional {
                type_id: optional_id,
                value: None,
            },
        };

        let some_rendered =
            render_core_instruction(&package_identity, &table, &routine, &some_instr)
                .expect("some");
        let nil_rendered =
            render_core_instruction(&package_identity, &table, &routine, &nil_instr).expect("nil");

        assert_eq!(
            some_rendered,
            "let l__pkg__entry__app__r12__l1__maybe = rt::FolOption::some(l__pkg__entry__app__r12__l0__value.clone());"
        );
        assert_eq!(
            nil_rendered,
            "let l__pkg__entry__app__r12__l2__empty = rt::FolOption::nil();"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_error_shell_construction() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let error_id = table.intern(fol_lower::LoweredType::Error {
            inner: Some(int_id),
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(13), "main", LoweredBlockId(0));
        let payload = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(error_id),
            recoverable_error_type: None,
            name: Some("error".to_string()),
        });
        let instruction = LoweredInstr {
            id: LoweredInstrId(27),
            result: Some(result),
            kind: LoweredInstrKind::ConstructError {
                type_id: error_id,
                value: Some(payload),
            },
        };

        let rendered = render_core_instruction(&package_identity, &table, &routine, &instruction)
            .expect("error shell");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r13__l1__error = rt::FolError::new(l__pkg__entry__app__r13__l0__value.clone());"
        );
    }

    #[test]
    fn runtime_shaped_instruction_rendering_emits_shell_unwraps_for_optional_and_error_values() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let optional_id = table.intern(fol_lower::LoweredType::Optional { inner: int_id });
        let error_id = table.intern(fol_lower::LoweredType::Error {
            inner: Some(int_id),
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(14), "main", LoweredBlockId(0));
        let maybe = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(optional_id),
            recoverable_error_type: None,
            name: Some("maybe".to_string()),
        });
        let err = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(error_id),
            recoverable_error_type: None,
            name: Some("err".to_string()),
        });
        let a = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("a".to_string()),
        });
        let b = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("b".to_string()),
        });
        let optional_instr = LoweredInstr {
            id: LoweredInstrId(28),
            result: Some(a),
            kind: LoweredInstrKind::UnwrapShell { operand: maybe },
        };
        let error_instr = LoweredInstr {
            id: LoweredInstrId(29),
            result: Some(b),
            kind: LoweredInstrKind::UnwrapShell { operand: err },
        };

        let optional_rendered =
            render_core_instruction(&package_identity, &table, &routine, &optional_instr)
                .expect("optional unwrap");
        let error_rendered =
            render_core_instruction(&package_identity, &table, &routine, &error_instr)
                .expect("error unwrap");

        assert_eq!(
            optional_rendered,
            "let l__pkg__entry__app__r14__l2__a = rt::unwrap_optional_shell(l__pkg__entry__app__r14__l0__maybe.clone()).expect(\"optional shell\");"
        );
        assert_eq!(
            error_rendered,
            "let l__pkg__entry__app__r14__l3__b = rt::unwrap_error_shell(l__pkg__entry__app__r14__l1__err.clone());"
        );
    }

    #[test]
    fn runtime_shaped_instruction_snapshot_stays_stable() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let optional_id = table.intern(fol_lower::LoweredType::Optional { inner: int_id });
        let error_id = table.intern(fol_lower::LoweredType::Error {
            inner: Some(int_id),
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(15), "main", LoweredBlockId(0));
        let value = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let rec = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: None,
            recoverable_error_type: None,
            name: Some("recover".to_string()),
        });
        let maybe = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(optional_id),
            recoverable_error_type: None,
            name: Some("maybe".to_string()),
        });
        let err = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(error_id),
            recoverable_error_type: None,
            name: Some("err".to_string()),
        });
        let count = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(4),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("count".to_string()),
        });
        let shown = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(5),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("shown".to_string()),
        });
        let failed = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(6),
            type_id: Some(bool_id),
            recoverable_error_type: None,
            name: Some("failed".to_string()),
        });
        let ok = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(7),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("ok".to_string()),
        });
        let bad = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(8),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("bad".to_string()),
        });

        let rendered = [
            LoweredInstr {
                id: LoweredInstrId(30),
                result: Some(count),
                kind: LoweredInstrKind::LengthOf { operand: maybe },
            },
            LoweredInstr {
                id: LoweredInstrId(31),
                result: Some(shown),
                kind: LoweredInstrKind::RuntimeHook {
                    intrinsic: intrinsic_by_canonical_name("echo").expect("echo").id,
                    args: vec![value],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(32),
                result: Some(failed),
                kind: LoweredInstrKind::CheckRecoverable { operand: rec },
            },
            LoweredInstr {
                id: LoweredInstrId(33),
                result: Some(ok),
                kind: LoweredInstrKind::UnwrapRecoverable { operand: rec },
            },
            LoweredInstr {
                id: LoweredInstrId(34),
                result: Some(bad),
                kind: LoweredInstrKind::ExtractRecoverableError { operand: rec },
            },
            LoweredInstr {
                id: LoweredInstrId(35),
                result: Some(maybe),
                kind: LoweredInstrKind::ConstructOptional {
                    type_id: optional_id,
                    value: Some(value),
                },
            },
            LoweredInstr {
                id: LoweredInstrId(36),
                result: Some(err),
                kind: LoweredInstrKind::ConstructError {
                    type_id: error_id,
                    value: Some(value),
                },
            },
            LoweredInstr {
                id: LoweredInstrId(37),
                result: Some(ok),
                kind: LoweredInstrKind::UnwrapShell { operand: maybe },
            },
        ]
        .iter()
        .map(|instruction| {
            render_core_instruction(&package_identity, &table, &routine, instruction)
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("runtime snapshot should render")
        .join("\n");

        assert_eq!(
            rendered,
            concat!(
                "let l__pkg__entry__app__r15__l4__count = rt::len(&l__pkg__entry__app__r15__l2__maybe);\n",
                "let l__pkg__entry__app__r15__l5__shown = rt::echo(l__pkg__entry__app__r15__l0__value);\n",
                "let l__pkg__entry__app__r15__l6__failed = rt::check_recoverable(&l__pkg__entry__app__r15__l1__recover);\n",
                "let l__pkg__entry__app__r15__l7__ok = l__pkg__entry__app__r15__l1__recover.clone().into_value().expect(\"recoverable success\");\n",
                "let l__pkg__entry__app__r15__l8__bad = l__pkg__entry__app__r15__l1__recover.clone().into_error().expect(\"recoverable error\");\n",
                "let l__pkg__entry__app__r15__l2__maybe = rt::FolOption::some(l__pkg__entry__app__r15__l0__value.clone());\n",
                "let l__pkg__entry__app__r15__l3__err = rt::FolError::new(l__pkg__entry__app__r15__l0__value.clone());\n",
                "let l__pkg__entry__app__r15__l7__ok = rt::unwrap_optional_shell(l__pkg__entry__app__r15__l2__maybe.clone()).expect(\"optional shell\");"
            )
        );
    }

    #[test]
    fn aggregate_and_container_rendering_emits_native_array_literals() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let array_id = table.intern(fol_lower::LoweredType::Array {
            element_type: int_id,
            size: Some(2),
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(16), "main", LoweredBlockId(0));
        let a = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("a".to_string()),
        });
        let b = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("b".to_string()),
        });
        let result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(array_id),
            recoverable_error_type: None,
            name: Some("arr".to_string()),
        });
        let instruction = LoweredInstr {
            id: LoweredInstrId(40),
            result: Some(result),
            kind: LoweredInstrKind::ConstructLinear {
                kind: LoweredLinearKind::Array,
                type_id: array_id,
                elements: vec![a, b],
            },
        };

        let rendered = render_core_instruction(&package_identity, &table, &routine, &instruction)
            .expect("array");

        assert_eq!(
            rendered,
            "let l__pkg__entry__app__r16__l2__arr = [l__pkg__entry__app__r16__l0__a.clone(), l__pkg__entry__app__r16__l1__b.clone()];"
        );
    }

    #[test]
    fn aggregate_and_container_rendering_emits_vector_and_sequence_runtime_constructors() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let vec_id = table.intern(fol_lower::LoweredType::Vector {
            element_type: int_id,
        });
        let seq_id = table.intern(fol_lower::LoweredType::Sequence {
            element_type: int_id,
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(17), "main", LoweredBlockId(0));
        let a = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("a".to_string()),
        });
        let b = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("b".to_string()),
        });
        let vec_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(vec_id),
            recoverable_error_type: None,
            name: Some("vec".to_string()),
        });
        let seq_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(seq_id),
            recoverable_error_type: None,
            name: Some("seq".to_string()),
        });
        let vec_instr = LoweredInstr {
            id: LoweredInstrId(41),
            result: Some(vec_result),
            kind: LoweredInstrKind::ConstructLinear {
                kind: LoweredLinearKind::Vector,
                type_id: vec_id,
                elements: vec![a, b],
            },
        };
        let seq_instr = LoweredInstr {
            id: LoweredInstrId(42),
            result: Some(seq_result),
            kind: LoweredInstrKind::ConstructLinear {
                kind: LoweredLinearKind::Sequence,
                type_id: seq_id,
                elements: vec![a, b],
            },
        };

        let vec_rendered = render_core_instruction(&package_identity, &table, &routine, &vec_instr)
            .expect("vector");
        let seq_rendered = render_core_instruction(&package_identity, &table, &routine, &seq_instr)
            .expect("sequence");

        assert_eq!(
            vec_rendered,
            "let l__pkg__entry__app__r17__l2__vec = rt::FolVec::from_items(vec![l__pkg__entry__app__r17__l0__a.clone(), l__pkg__entry__app__r17__l1__b.clone()]);"
        );
        assert_eq!(
            seq_rendered,
            "let l__pkg__entry__app__r17__l3__seq = rt::FolSeq::from_items(vec![l__pkg__entry__app__r17__l0__a.clone(), l__pkg__entry__app__r17__l1__b.clone()]);"
        );
    }

    #[test]
    fn aggregate_and_container_rendering_emits_set_and_map_runtime_constructors() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let set_id = table.intern(fol_lower::LoweredType::Set {
            member_types: vec![int_id],
        });
        let map_id = table.intern(fol_lower::LoweredType::Map {
            key_type: int_id,
            value_type: int_id,
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(18), "main", LoweredBlockId(0));
        let a = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("a".to_string()),
        });
        let b = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("b".to_string()),
        });
        let set_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(set_id),
            recoverable_error_type: None,
            name: Some("set".to_string()),
        });
        let map_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(map_id),
            recoverable_error_type: None,
            name: Some("map".to_string()),
        });
        let set_instr = LoweredInstr {
            id: LoweredInstrId(43),
            result: Some(set_result),
            kind: LoweredInstrKind::ConstructSet {
                type_id: set_id,
                members: vec![a, b],
            },
        };
        let map_instr = LoweredInstr {
            id: LoweredInstrId(44),
            result: Some(map_result),
            kind: LoweredInstrKind::ConstructMap {
                type_id: map_id,
                entries: vec![(a, b), (b, a)],
            },
        };

        let set_rendered =
            render_core_instruction(&package_identity, &table, &routine, &set_instr).expect("set");
        let map_rendered =
            render_core_instruction(&package_identity, &table, &routine, &map_instr).expect("map");

        assert_eq!(
            set_rendered,
            "let l__pkg__entry__app__r18__l2__set = rt::FolSet::from_items(vec![l__pkg__entry__app__r18__l0__a.clone(), l__pkg__entry__app__r18__l1__b.clone()]);"
        );
        assert_eq!(
            map_rendered,
            "let l__pkg__entry__app__r18__l3__map = rt::FolMap::from_pairs(vec![(l__pkg__entry__app__r18__l0__a.clone(), l__pkg__entry__app__r18__l1__b.clone()), (l__pkg__entry__app__r18__l1__b.clone(), l__pkg__entry__app__r18__l0__a.clone())]);"
        );
    }

    #[test]
    fn aggregate_and_container_rendering_emits_runtime_index_helpers() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let array_id = table.intern(fol_lower::LoweredType::Array {
            element_type: int_id,
            size: Some(2),
        });
        let vec_id = table.intern(fol_lower::LoweredType::Vector {
            element_type: int_id,
        });
        let seq_id = table.intern(fol_lower::LoweredType::Sequence {
            element_type: int_id,
        });
        let map_id = table.intern(fol_lower::LoweredType::Map {
            key_type: int_id,
            value_type: int_id,
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(19), "main", LoweredBlockId(0));
        let array = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(array_id),
            recoverable_error_type: None,
            name: Some("arr".to_string()),
        });
        let vector = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(vec_id),
            recoverable_error_type: None,
            name: Some("vec".to_string()),
        });
        let sequence = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(seq_id),
            recoverable_error_type: None,
            name: Some("seq".to_string()),
        });
        let map = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(map_id),
            recoverable_error_type: None,
            name: Some("map".to_string()),
        });
        let index = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(4),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("index".to_string()),
        });
        let arr_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(5),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("a".to_string()),
        });
        let vec_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(6),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("b".to_string()),
        });
        let seq_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(7),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("c".to_string()),
        });
        let map_result = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(8),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("d".to_string()),
        });

        let rendered = [
            LoweredInstr {
                id: LoweredInstrId(45),
                result: Some(arr_result),
                kind: LoweredInstrKind::IndexAccess {
                    container: array,
                    index,
                },
            },
            LoweredInstr {
                id: LoweredInstrId(46),
                result: Some(vec_result),
                kind: LoweredInstrKind::IndexAccess {
                    container: vector,
                    index,
                },
            },
            LoweredInstr {
                id: LoweredInstrId(47),
                result: Some(seq_result),
                kind: LoweredInstrKind::IndexAccess {
                    container: sequence,
                    index,
                },
            },
            LoweredInstr {
                id: LoweredInstrId(48),
                result: Some(map_result),
                kind: LoweredInstrKind::IndexAccess {
                    container: map,
                    index,
                },
            },
        ]
        .iter()
        .map(|instruction| {
            render_core_instruction(&package_identity, &table, &routine, instruction)
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("index renders");

        assert_eq!(
            rendered[0],
            "let l__pkg__entry__app__r19__l5__a = rt::index_array(&l__pkg__entry__app__r19__l0__arr, l__pkg__entry__app__r19__l4__index.clone()).expect(\"array index\").clone();"
        );
        assert_eq!(
            rendered[1],
            "let l__pkg__entry__app__r19__l6__b = rt::index_vec(&l__pkg__entry__app__r19__l1__vec, l__pkg__entry__app__r19__l4__index.clone()).expect(\"vector index\").clone();"
        );
        assert_eq!(
            rendered[2],
            "let l__pkg__entry__app__r19__l7__c = rt::index_seq(&l__pkg__entry__app__r19__l2__seq, l__pkg__entry__app__r19__l4__index.clone()).expect(\"sequence index\").clone();"
        );
        assert_eq!(
            rendered[3],
            "let l__pkg__entry__app__r19__l8__d = rt::lookup_map(&l__pkg__entry__app__r19__l3__map, &l__pkg__entry__app__r19__l4__index).expect(\"map key\").clone();"
        );
    }

    #[test]
    fn aggregate_and_container_rendering_emits_record_and_entry_constructors() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let record_type = table.intern(LoweredType::Record {
            fields: BTreeMap::from([("count".to_string(), int_id)]),
        });
        let entry_type = table.intern(LoweredType::Entry {
            variants: BTreeMap::from([("Ok".to_string(), Some(int_id))]),
        });

        let mut package =
            LoweredPackage::new(fol_lower::LoweredPackageId(0), package_identity.clone());
        package.source_units.push(fol_lower::LoweredSourceUnit {
            source_unit_id: SourceUnitId(0),
            path: "app/main.fol".to_string(),
            package: "app".to_string(),
            namespace: "app".to_string(),
        });
        package.type_decls.insert(
            SymbolId(0),
            LoweredTypeDecl {
                symbol_id: SymbolId(0),
                source_unit_id: SourceUnitId(0),
                name: "Counter".to_string(),
                runtime_type: record_type,
                kind: LoweredTypeDeclKind::Record {
                    fields: vec![LoweredFieldLayout {
                        name: "count".to_string(),
                        type_id: int_id,
                    }],
                },
            },
        );
        package.type_decls.insert(
            SymbolId(1),
            LoweredTypeDecl {
                symbol_id: SymbolId(1),
                source_unit_id: SourceUnitId(0),
                name: "Status".to_string(),
                runtime_type: entry_type,
                kind: LoweredTypeDeclKind::Entry {
                    variants: vec![LoweredVariantLayout {
                        name: "Ok".to_string(),
                        payload_type: Some(int_id),
                    }],
                },
            },
        );
        let workspace = LoweredWorkspace::new(
            package_identity.clone(),
            BTreeMap::from([(package_identity.clone(), package)]),
            Vec::new(),
            table.clone(),
            LoweredSourceMap::new(),
            LoweredRecoverableAbi::v1(bool_id),
        );

        let mut routine = LoweredRoutine::new(LoweredRoutineId(30), "main", LoweredBlockId(0));
        let value = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });
        let record_out = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(record_type),
            recoverable_error_type: None,
            name: Some("record".to_string()),
        });
        let entry_out = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(entry_type),
            recoverable_error_type: None,
            name: Some("entry".to_string()),
        });

        let record_rendered = render_core_instruction_in_workspace(
            Some(&workspace),
            &package_identity,
            &table,
            &routine,
            &LoweredInstr {
                id: LoweredInstrId(70),
                result: Some(record_out),
                kind: LoweredInstrKind::ConstructRecord {
                    type_id: record_type,
                    fields: vec![("count".to_string(), value)],
                },
            },
        )
        .expect("record constructor");
        let entry_rendered = render_core_instruction_in_workspace(
            Some(&workspace),
            &package_identity,
            &table,
            &routine,
            &LoweredInstr {
                id: LoweredInstrId(71),
                result: Some(entry_out),
                kind: LoweredInstrKind::ConstructEntry {
                    type_id: entry_type,
                    variant: "Ok".to_string(),
                    payload: Some(value),
                },
            },
        )
        .expect("entry constructor");

        assert_eq!(
            record_rendered,
            "let l__pkg__entry__app__r30__l1__record = crate::packages::pkg__entry__app::root::ty__pkg__entry__app__t2__counter { count: l__pkg__entry__app__r30__l0__value.clone() };"
        );
        assert_eq!(
            entry_rendered,
            "let l__pkg__entry__app__r30__l2__entry = crate::packages::pkg__entry__app::root::ty__pkg__entry__app__t3__status::Ok(l__pkg__entry__app__r30__l0__value.clone());"
        );
    }

    #[test]
    fn aggregate_and_container_snapshot_stays_stable() {
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let array_id = table.intern(fol_lower::LoweredType::Array {
            element_type: int_id,
            size: Some(2),
        });
        let vec_id = table.intern(fol_lower::LoweredType::Vector {
            element_type: int_id,
        });
        let seq_id = table.intern(fol_lower::LoweredType::Sequence {
            element_type: int_id,
        });
        let set_id = table.intern(fol_lower::LoweredType::Set {
            member_types: vec![int_id],
        });
        let map_id = table.intern(fol_lower::LoweredType::Map {
            key_type: int_id,
            value_type: int_id,
        });
        let mut routine = LoweredRoutine::new(LoweredRoutineId(20), "main", LoweredBlockId(0));
        let a = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("a".to_string()),
        });
        let b = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("b".to_string()),
        });
        let arr = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(array_id),
            recoverable_error_type: None,
            name: Some("arr".to_string()),
        });
        let vec = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(3),
            type_id: Some(vec_id),
            recoverable_error_type: None,
            name: Some("vec".to_string()),
        });
        let seq = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(4),
            type_id: Some(seq_id),
            recoverable_error_type: None,
            name: Some("seq".to_string()),
        });
        let set = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(5),
            type_id: Some(set_id),
            recoverable_error_type: None,
            name: Some("set".to_string()),
        });
        let map = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(6),
            type_id: Some(map_id),
            recoverable_error_type: None,
            name: Some("map".to_string()),
        });
        let out = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(7),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("out".to_string()),
        });

        let rendered = [
            LoweredInstr {
                id: LoweredInstrId(49),
                result: Some(arr),
                kind: LoweredInstrKind::ConstructLinear {
                    kind: LoweredLinearKind::Array,
                    type_id: array_id,
                    elements: vec![a, b],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(50),
                result: Some(vec),
                kind: LoweredInstrKind::ConstructLinear {
                    kind: LoweredLinearKind::Vector,
                    type_id: vec_id,
                    elements: vec![a, b],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(51),
                result: Some(seq),
                kind: LoweredInstrKind::ConstructLinear {
                    kind: LoweredLinearKind::Sequence,
                    type_id: seq_id,
                    elements: vec![a, b],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(52),
                result: Some(set),
                kind: LoweredInstrKind::ConstructSet {
                    type_id: set_id,
                    members: vec![a, b],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(53),
                result: Some(map),
                kind: LoweredInstrKind::ConstructMap {
                    type_id: map_id,
                    entries: vec![(a, b)],
                },
            },
            LoweredInstr {
                id: LoweredInstrId(54),
                result: Some(out),
                kind: LoweredInstrKind::IndexAccess {
                    container: vec,
                    index: a,
                },
            },
        ]
        .iter()
        .map(|instruction| {
            render_core_instruction(&package_identity, &table, &routine, instruction)
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("container snapshot renders")
        .join("\n");

        assert_eq!(
            rendered,
            concat!(
                "let l__pkg__entry__app__r20__l2__arr = [l__pkg__entry__app__r20__l0__a.clone(), l__pkg__entry__app__r20__l1__b.clone()];\n",
                "let l__pkg__entry__app__r20__l3__vec = rt::FolVec::from_items(vec![l__pkg__entry__app__r20__l0__a.clone(), l__pkg__entry__app__r20__l1__b.clone()]);\n",
                "let l__pkg__entry__app__r20__l4__seq = rt::FolSeq::from_items(vec![l__pkg__entry__app__r20__l0__a.clone(), l__pkg__entry__app__r20__l1__b.clone()]);\n",
                "let l__pkg__entry__app__r20__l5__set = rt::FolSet::from_items(vec![l__pkg__entry__app__r20__l0__a.clone(), l__pkg__entry__app__r20__l1__b.clone()]);\n",
                "let l__pkg__entry__app__r20__l6__map = rt::FolMap::from_pairs(vec![(l__pkg__entry__app__r20__l0__a.clone(), l__pkg__entry__app__r20__l1__b.clone())]);\n",
                "let l__pkg__entry__app__r20__l7__out = rt::index_vec(&l__pkg__entry__app__r20__l3__vec, l__pkg__entry__app__r20__l0__a.clone()).expect(\"vector index\").clone();"
            )
        );
    }

    #[test]
    fn unsupported_lowered_instruction_families_fail_explicitly() {
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let package_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
        let mut routine = LoweredRoutine::new(LoweredRoutineId(21), "main", LoweredBlockId(0));
        let local_id = routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(int_id),
            recoverable_error_type: None,
            name: Some("value".to_string()),
        });

        let unsupported = [LoweredInstr {
            id: LoweredInstrId(62),
            result: Some(local_id),
            kind: LoweredInstrKind::Cast {
                operand: local_id,
                target_type: int_id,
            },
        }];

        for instruction in unsupported {
            let error = render_core_instruction(&package_identity, &table, &routine, &instruction)
                .expect_err("unsupported families should fail explicitly");
            assert_eq!(error.kind(), crate::BackendErrorKind::Unsupported);
        }
    }
}

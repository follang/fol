use crate::{BackendError, BackendErrorKind, BackendResult};
use fol_lower::{LoweredBuiltinType, LoweredType, LoweredTypeId, LoweredTypeTable};

pub fn render_rust_type(type_table: &LoweredTypeTable, type_id: LoweredTypeId) -> BackendResult<String> {
    let Some(ty) = type_table.get(type_id) else {
        return Err(BackendError::new(
            BackendErrorKind::InvalidInput,
            format!("lowered type {:?} is missing from the type table", type_id),
        ));
    };

    match ty {
        LoweredType::Builtin(LoweredBuiltinType::Str) => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            "Rust type rendering for runtime-backed string values is not implemented yet",
        )),
        LoweredType::Builtin(builtin) => Ok(render_builtin_type(*builtin).to_string()),
        other => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            format!("Rust type rendering is not implemented yet for {other:?}"),
        )),
    }
}

fn render_builtin_type(builtin: LoweredBuiltinType) -> &'static str {
    match builtin {
        LoweredBuiltinType::Int => "rt::FolInt",
        LoweredBuiltinType::Float => "rt::FolFloat",
        LoweredBuiltinType::Bool => "rt::FolBool",
        LoweredBuiltinType::Char => "rt::FolChar",
        LoweredBuiltinType::Never => "rt::FolNever",
        LoweredBuiltinType::Str => unreachable!("string mapping lands in the runtime-backed phase"),
    }
}

#[cfg(test)]
mod tests {
    use super::render_rust_type;
    use fol_lower::{LoweredBuiltinType, LoweredTypeTable};

    #[test]
    fn builtin_scalar_type_rendering_uses_backend_owned_runtime_aliases() {
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let float_id = table.intern_builtin(LoweredBuiltinType::Float);
        let bool_id = table.intern_builtin(LoweredBuiltinType::Bool);
        let char_id = table.intern_builtin(LoweredBuiltinType::Char);
        let never_id = table.intern_builtin(LoweredBuiltinType::Never);

        assert_eq!(render_rust_type(&table, int_id), Ok("rt::FolInt".to_string()));
        assert_eq!(render_rust_type(&table, float_id), Ok("rt::FolFloat".to_string()));
        assert_eq!(render_rust_type(&table, bool_id), Ok("rt::FolBool".to_string()));
        assert_eq!(render_rust_type(&table, char_id), Ok("rt::FolChar".to_string()));
        assert_eq!(render_rust_type(&table, never_id), Ok("rt::FolNever".to_string()));
    }
}

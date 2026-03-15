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
        LoweredType::Builtin(LoweredBuiltinType::Str) => Ok("rt::FolStr".to_string()),
        LoweredType::Builtin(builtin) => Ok(render_builtin_type(*builtin).to_string()),
        LoweredType::Array {
            element_type,
            size: Some(size),
        } => Ok(format!(
            "rt::FolArray<{}, {size}>",
            render_rust_type(type_table, *element_type)?
        )),
        LoweredType::Array { size: None, .. } => Err(BackendError::new(
            BackendErrorKind::Unsupported,
            "Rust type rendering for unsized arrays is not implemented yet",
        )),
        LoweredType::Vector { element_type } => Ok(format!(
            "rt::FolVec<{}>",
            render_rust_type(type_table, *element_type)?
        )),
        LoweredType::Sequence { element_type } => Ok(format!(
            "rt::FolSeq<{}>",
            render_rust_type(type_table, *element_type)?
        )),
        LoweredType::Set { member_types } => match member_types.as_slice() {
            [member_type] => Ok(format!(
                "rt::FolSet<{}>",
                render_rust_type(type_table, *member_type)?
            )),
            _ => Err(BackendError::new(
                BackendErrorKind::Unsupported,
                "Rust type rendering for heterogeneous set members is not implemented yet",
            )),
        },
        LoweredType::Map {
            key_type,
            value_type,
        } => Ok(format!(
            "rt::FolMap<{}, {}>",
            render_rust_type(type_table, *key_type)?,
            render_rust_type(type_table, *value_type)?
        )),
        LoweredType::Optional { inner } => Ok(format!(
            "rt::FolOption<{}>",
            render_rust_type(type_table, *inner)?
        )),
        LoweredType::Error { inner } => Ok(match inner {
            Some(inner) => format!("rt::FolError<{}>", render_rust_type(type_table, *inner)?),
            None => "rt::FolError<()>".to_string(),
        }),
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
    use fol_lower::{LoweredBuiltinType, LoweredType, LoweredTypeTable};

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

    #[test]
    fn runtime_backed_type_rendering_covers_current_v1_families() {
        let mut table = LoweredTypeTable::new();
        let int_id = table.intern_builtin(LoweredBuiltinType::Int);
        let str_id = table.intern_builtin(LoweredBuiltinType::Str);
        let array_id = table.intern(LoweredType::Array {
            element_type: int_id,
            size: Some(3),
        });
        let vec_id = table.intern(LoweredType::Vector { element_type: int_id });
        let seq_id = table.intern(LoweredType::Sequence { element_type: str_id });
        let set_id = table.intern(LoweredType::Set {
            member_types: vec![int_id],
        });
        let map_id = table.intern(LoweredType::Map {
            key_type: str_id,
            value_type: int_id,
        });
        let option_id = table.intern(LoweredType::Optional { inner: str_id });
        let error_id = table.intern(LoweredType::Error { inner: Some(str_id) });

        assert_eq!(render_rust_type(&table, str_id), Ok("rt::FolStr".to_string()));
        assert_eq!(
            render_rust_type(&table, array_id),
            Ok("rt::FolArray<rt::FolInt, 3>".to_string())
        );
        assert_eq!(render_rust_type(&table, vec_id), Ok("rt::FolVec<rt::FolInt>".to_string()));
        assert_eq!(render_rust_type(&table, seq_id), Ok("rt::FolSeq<rt::FolStr>".to_string()));
        assert_eq!(render_rust_type(&table, set_id), Ok("rt::FolSet<rt::FolInt>".to_string()));
        assert_eq!(
            render_rust_type(&table, map_id),
            Ok("rt::FolMap<rt::FolStr, rt::FolInt>".to_string())
        );
        assert_eq!(
            render_rust_type(&table, option_id),
            Ok("rt::FolOption<rt::FolStr>".to_string())
        );
        assert_eq!(
            render_rust_type(&table, error_id),
            Ok("rt::FolError<rt::FolStr>".to_string())
        );
    }
}

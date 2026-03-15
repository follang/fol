use crate::{mangle_global_name, render_rust_type, BackendResult};
use fol_lower::{LoweredGlobal, LoweredTypeTable};
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

#[cfg(test)]
mod tests {
    use super::render_global_declaration;
    use crate::testing::package_identity;
    use fol_lower::{LoweredBuiltinType, LoweredGlobal, LoweredGlobalId, LoweredTypeTable};
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
}

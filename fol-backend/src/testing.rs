use fol_lower::{
    LoweredBuiltinType, LoweredEntryCandidate, LoweredExportMount, LoweredFieldLayout,
    LoweredGlobal, LoweredPackage, LoweredRecoverableAbi, LoweredRoutine, LoweredRoutineType,
    LoweredSourceMap, LoweredSourceMapEntry, LoweredSourceSymbol, LoweredSourceUnit,
    LoweredType, LoweredTypeDecl, LoweredTypeDeclKind, LoweredTypeTable, LoweredWorkspace,
};
use fol_parser::ast::SyntaxOrigin;
use fol_resolver::{PackageIdentity, PackageSourceKind, SourceUnitId, SymbolId};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn sample_lowered_workspace() -> LoweredWorkspace {
    let entry_identity = package_identity("app", PackageSourceKind::Entry, "/workspace/app");
    let shared_identity =
        package_identity("shared", PackageSourceKind::Local, "/workspace/shared");

    let mut type_table = LoweredTypeTable::new();
    let int_type = type_table.intern_builtin(LoweredBuiltinType::Int);
    let bool_type = type_table.intern_builtin(LoweredBuiltinType::Bool);
    let str_type = type_table.intern_builtin(LoweredBuiltinType::Str);

    let user_record_type = type_table.intern(LoweredType::Record {
        fields: BTreeMap::from([("name".to_string(), str_type), ("active".to_string(), bool_type)]),
    });
    let main_signature = type_table.intern(LoweredType::Routine(LoweredRoutineType {
        params: vec![bool_type],
        return_type: Some(int_type),
        error_type: Some(str_type),
    }));
    let helper_signature = type_table.intern(LoweredType::Routine(LoweredRoutineType {
        params: vec![int_type],
        return_type: Some(int_type),
        error_type: None,
    }));

    let mut entry_package = LoweredPackage::new(fol_lower::LoweredPackageId(0), entry_identity.clone());
    entry_package.source_units = vec![
        LoweredSourceUnit {
            source_unit_id: SourceUnitId(0),
            path: "app/main.fol".to_string(),
            package: "app".to_string(),
            namespace: "app".to_string(),
        },
        LoweredSourceUnit {
            source_unit_id: SourceUnitId(1),
            path: "app/math/add.fol".to_string(),
            package: "app".to_string(),
            namespace: "app::math".to_string(),
        },
    ];
    entry_package.exports = vec![LoweredExportMount {
        source_namespace: "app".to_string(),
        mounted_namespace_suffix: None,
    }];
    entry_package.types = vec![user_record_type];
    entry_package.routines = vec![fol_lower::LoweredRoutineId(0), fol_lower::LoweredRoutineId(1)];
    entry_package.globals = vec![fol_lower::LoweredGlobalId(0)];
    entry_package.type_decls.insert(
        SymbolId(10),
        LoweredTypeDecl {
            symbol_id: SymbolId(10),
            source_unit_id: SourceUnitId(0),
            name: "User".to_string(),
            runtime_type: user_record_type,
            kind: LoweredTypeDeclKind::Record {
                fields: vec![
                    LoweredFieldLayout {
                        name: "name".to_string(),
                        type_id: str_type,
                    },
                    LoweredFieldLayout {
                        name: "active".to_string(),
                        type_id: bool_type,
                    },
                ],
            },
        },
    );
    entry_package.global_decls.insert(
        fol_lower::LoweredGlobalId(0),
        LoweredGlobal {
            id: fol_lower::LoweredGlobalId(0),
            symbol_id: SymbolId(20),
            source_unit_id: SourceUnitId(0),
            name: "default_name".to_string(),
            type_id: str_type,
            recoverable_error_type: None,
            mutable: false,
        },
    );
    entry_package.routine_signatures.insert(SymbolId(30), main_signature);
    entry_package.routine_signatures.insert(SymbolId(31), helper_signature);
    entry_package.routine_decls.insert(
        fol_lower::LoweredRoutineId(0),
        routine(fol_lower::LoweredRoutineId(0), "main", Some(SymbolId(30)), Some(SourceUnitId(0)), Some(main_signature)),
    );
    entry_package.routine_decls.insert(
        fol_lower::LoweredRoutineId(1),
        routine(
            fol_lower::LoweredRoutineId(1),
            "add_one",
            Some(SymbolId(31)),
            Some(SourceUnitId(1)),
            Some(helper_signature),
        ),
    );

    let mut shared_package =
        LoweredPackage::new(fol_lower::LoweredPackageId(1), shared_identity.clone());
    shared_package.source_units = vec![
        LoweredSourceUnit {
            source_unit_id: SourceUnitId(2),
            path: "shared/lib.fol".to_string(),
            package: "shared".to_string(),
            namespace: "shared".to_string(),
        },
        LoweredSourceUnit {
            source_unit_id: SourceUnitId(3),
            path: "shared/util/log.fol".to_string(),
            package: "shared".to_string(),
            namespace: "shared::util".to_string(),
        },
    ];
    shared_package.exports = vec![LoweredExportMount {
        source_namespace: "shared::util".to_string(),
        mounted_namespace_suffix: Some("util".to_string()),
    }];
    shared_package.globals = vec![fol_lower::LoweredGlobalId(1)];
    shared_package.routines = vec![fol_lower::LoweredRoutineId(2)];
    shared_package.global_decls.insert(
        fol_lower::LoweredGlobalId(1),
        LoweredGlobal {
            id: fol_lower::LoweredGlobalId(1),
            symbol_id: SymbolId(40),
            source_unit_id: SourceUnitId(2),
            name: "answer".to_string(),
            type_id: int_type,
            recoverable_error_type: None,
            mutable: false,
        },
    );
    shared_package.routine_signatures.insert(SymbolId(41), helper_signature);
    shared_package.routine_decls.insert(
        fol_lower::LoweredRoutineId(2),
        routine(
            fol_lower::LoweredRoutineId(2),
            "emit",
            Some(SymbolId(41)),
            Some(SourceUnitId(3)),
            Some(helper_signature),
        ),
    );

    let mut packages = BTreeMap::new();
    packages.insert(entry_identity.clone(), entry_package);
    packages.insert(shared_identity.clone(), shared_package);

    let mut source_map = LoweredSourceMap::new();
    source_map.push(source_map_entry(
        "app/main.fol",
        1,
        1,
        3,
        LoweredSourceSymbol::Routine(fol_lower::LoweredRoutineId(0)),
    ));
    source_map.push(source_map_entry(
        "shared/lib.fol",
        2,
        1,
        6,
        LoweredSourceSymbol::Global(fol_lower::LoweredGlobalId(1)),
    ));

    LoweredWorkspace::new(
        entry_identity.clone(),
        packages,
        vec![LoweredEntryCandidate {
            package_identity: entry_identity,
            routine_id: fol_lower::LoweredRoutineId(0),
            name: "main".to_string(),
        }],
        type_table,
        source_map,
        LoweredRecoverableAbi::v1(bool_type),
    )
}

pub(crate) fn package_identity(
    name: &str,
    kind: PackageSourceKind,
    root: &str,
) -> PackageIdentity {
    PackageIdentity {
        source_kind: kind,
        canonical_root: root.to_string(),
        display_name: name.to_string(),
    }
}

pub(crate) fn distinct_namespaces(workspace: &LoweredWorkspace) -> BTreeSet<String> {
    workspace
        .packages()
        .flat_map(|package| package.source_units.iter().map(|unit| unit.namespace.clone()))
        .collect()
}

fn routine(
    id: fol_lower::LoweredRoutineId,
    name: &str,
    symbol_id: Option<SymbolId>,
    source_unit_id: Option<SourceUnitId>,
    signature: Option<fol_lower::LoweredTypeId>,
) -> LoweredRoutine {
    let mut routine = LoweredRoutine::new(id, name, fol_lower::LoweredBlockId(0));
    routine.symbol_id = symbol_id;
    routine.source_unit_id = source_unit_id;
    routine.signature = signature;
    routine
}

fn source_map_entry(
    file: &str,
    line: usize,
    column: usize,
    length: usize,
    symbol: LoweredSourceSymbol,
) -> LoweredSourceMapEntry {
    LoweredSourceMapEntry {
        symbol,
        origin: SyntaxOrigin {
            file: Some(file.to_string()),
            line,
            column,
            length,
        },
    }
}
